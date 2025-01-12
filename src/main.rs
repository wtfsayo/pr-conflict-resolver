use git2::{Cred, FetchOptions, PushOptions, RemoteCallbacks, Repository};
use octocrab::{models::pulls::PullRequest, Octocrab};
use std::env;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug)]
struct PRManager {
    github: Octocrab,
    token: String,
    repo_owner: String,
    repo_name: String,
    base_branch: String,
}

impl PRManager {
    fn new(
        token: String,
        repo_owner: String,
        repo_name: String,
        base_branch: String,
    ) -> Result<Self, Box<dyn Error>> {
        let github = Octocrab::builder().personal_token(token.clone()).build()?;

        Ok(PRManager {
            github,
            token,
            repo_owner,
            repo_name,
            base_branch,
        })
    }

    fn setup_git_config(&self) -> Result<(), Box<dyn Error>> {
        Command::new("git")
            .args(&["config", "--global", "credential.helper", "store"])
            .output()?;

        let credentials_path = dirs::home_dir()
            .ok_or("Could not find home directory")?
            .join(".git-credentials");

        fs::write(
            credentials_path,
            format!("https://{}:x-oauth-basic@github.com\n", self.token),
        )?;

        Ok(())
    }

    async fn clone_and_setup_repo(
        &self,
        pr_number: u64,
    ) -> Result<(Repository, PullRequest, String), Box<dyn Error>> {
        // Get PR information
        let pr = self
            .github
            .pulls(self.repo_owner.clone(), self.repo_name.clone())
            .get(pr_number)
            .await?;

        let source_branch = pr.head.ref_field.clone();
        let source_repo = pr.head.repo.as_ref().unwrap().full_name.as_ref();

        // Remove existing temp repo if it exists
        let temp_path = Path::new("temp_repo");
        if temp_path.exists() {
            fs::remove_dir_all(temp_path)?;
        }

        // Setup authentication callbacks
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, _username, _allowed| {
            Cred::userpass_plaintext("token", &self.token)
        });

        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        // Clone repository
        let repo = Repository::clone(
            &format!(
                "https://github.com/{}/{}.git",
                self.repo_owner, self.repo_name
            ),
            temp_path,
        )?;

        // Add source repository as remote if it's a fork
        if source_repo.map(|s| s.as_str()).unwrap_or_default() != format!("{}/{}", self.repo_owner, self.repo_name) {
            repo.remote(
                "source",
                &format!("https://github.com/{}.git", source_repo.unwrap()),
            )?;

            let mut remote = repo.find_remote("source")?;
            remote.fetch(
                &["refs/heads/*:refs/remotes/source/*"],
                Some(&mut fetch_options),
                None,
            )?;
        }

        Ok((repo, pr, source_branch))
    }

    fn merge_base_branch(&self, repo: &Repository) -> Result<bool, Box<dyn Error>> {
        // Fetch all remotes
        let mut remote = repo.find_remote("origin")?;
        remote.fetch(&["refs/heads/*:refs/remotes/origin/*"], None, None)?;

        // Try to find and checkout base branch
        let base_branch_ref = repo
            .find_branch(
                &format!("origin/{}", self.base_branch),
                git2::BranchType::Remote,
            )
            .or_else(|_| {
                repo.find_branch(
                    &format!("source/{}", self.base_branch),
                    git2::BranchType::Remote,
                )
            })?;

        let base_commit = base_branch_ref.get().peel_to_commit()?;

        // Create and checkout temporary branch
        let temp_branch = repo.branch("temp_branch", &base_commit, true)?;
        repo.checkout_tree(temp_branch.get().peel_to_tree()?.as_object(), None)?;
        repo.set_head("refs/heads/temp_branch")?;

        // Fix the merge by creating an annotated commit
        let annotated_commit =
            repo.reference_to_annotated_commit(&base_branch_ref.into_reference())?;
        match repo.merge(&[&annotated_commit], None, None) {
            Ok(_) => Ok(true),
            Err(e) => {
                println!("Merge conflict occurred: {}", e);
                repo.cleanup_state()?;
                Ok(false)
            }
        }
    }

    async fn create_new_pr(
        &self,
        original_pr: &PullRequest,
        new_branch_name: &str,
    ) -> Result<PullRequest, Box<dyn Error>> {
        let new_body = format!(
            "This is a reposted PR originally created by @{}\n\nOriginal PR: #{}\n\n---\n{}",
            original_pr.user.as_ref().unwrap().login,
            original_pr.number,
            original_pr.body.as_deref().unwrap_or("")
        );

        let new_pr = self
            .github
            .pulls(self.repo_owner.clone(), self.repo_name.clone())
            .create(
                format!(
                    "[Repost] {}",
                    original_pr.title.as_ref().unwrap_or(&String::new())
                ),
                new_branch_name.to_string(),
                self.base_branch.clone(),
            )
            .body(new_body)
            .send()
            .await?;

        // Copy labels would go here
        // Note: Label copying requires additional implementation

        Ok(new_pr)
    }

    async fn process_pr(
        &self,
        pr_number: u64,
        _interactive: bool,
    ) -> Result<String, Box<dyn Error>> {
        self.setup_git_config()?;
        let (repo, original_pr, _source_branch) = self.clone_and_setup_repo(pr_number).await?;

        let new_branch_name = format!("pr{}_fix", pr_number);

        if !self.merge_base_branch(&repo)? {
            return Ok(format!(
                "Failed to merge {} branch. Manual intervention required.",
                self.base_branch
            ));
        }

        // Push to new branch
        let mut remote = repo.find_remote("origin")?;
        let mut push_options = PushOptions::new();
        remote.push(
            &[&format!(
                "refs/heads/temp_branch:refs/heads/{}",
                new_branch_name
            )],
            Some(&mut push_options),
        )?;

        let new_pr = self.create_new_pr(&original_pr, &new_branch_name).await?;

        Ok(format!(
            "Successfully created new PR: {}",
            new_pr.html_url.unwrap()
        ))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <pr_number> [--no-interactive]", args[0]);
        return Ok(());
    }

    let token = env::var("GITHUB_TOKEN")?;
    let repo_owner = env::var("REPO_OWNER")?;
    let repo_name = env::var("REPO_NAME")?;
    let base_branch = env::var("BASE_BRANCH").unwrap_or_else(|_| "develop".to_string());

    let pr_number = args[1].parse::<u64>()?;
    let interactive = !args[2..].contains(&"--no-interactive".to_string());

    let pr_manager = PRManager::new(token, repo_owner, repo_name, base_branch)?;
    let result = pr_manager.process_pr(pr_number, interactive).await?;
    println!("{}", result);

    Ok(())
}
