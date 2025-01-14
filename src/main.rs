use std::env;
use std::error::Error;
use std::process::Command;

fn run_command(cmd: &str, args: &[&str]) -> Result<(), Box<dyn Error>> {
    let status = Command::new(cmd).args(args).status()?;
    if !status.success() {
        return Err(format!("Command {:?} {:?} failed.", cmd, args).into());
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    // Parse CLI arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: pr-resolver <pr_number> [--no-interactive] [--work-dir <path>]");
        return Ok(());
    }

    let pr_number = args[1].parse::<u64>()?;

    // Optional flags (we won't use them much, but let's keep the structure)
    let mut _interactive = true;
    let mut _work_dir = None;
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--no-interactive" => {
                _interactive = false;
                i += 1;
            }
            "--work-dir" => {
                if i + 1 < args.len() {
                    _work_dir = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    println!("Error: --work-dir requires a path argument");
                    return Ok(());
                }
            }
            _ => i += 1,
        }
    }

    // Example local PR branch (adjust as needed):
    let pr_branch = format!("pull/{}/head", pr_number);
    // We'll name our "fork" branch:
    let fork_branch = format!("pr{}_fork", pr_number);

    // 1. Try checking out the PR branch directly
    let checkout_result = Command::new("git")
        .args(&["checkout", &pr_branch])
        .status();

    // 2. If checkout fails, create a new branch from that PR
    match checkout_result {
        Ok(status) if status.success() => {
            println!(
                "Checked out existing branch '{}'. Merging 'develop' by default.",
                pr_branch
            );
        }
        Ok(_) | Err(_) => {
            println!(
                "Could not checkout '{}'. Creating new branch '{}' from '{}'.",
                pr_branch, fork_branch, pr_branch
            );
            // fetch the remote reference using the correct GitHub PR reference format
            run_command(
                "git",
                &[
                    "fetch",
                    "origin",
                    &format!("refs/pull/{}/head:refs/remotes/origin/pr/{}", pr_number, pr_number),
                ],
            )?;
            // Create and checkout the fork branch from the fetched PR
            run_command(
                "git",
                &["checkout", "-b", &fork_branch, &format!("origin/pr/{}", pr_number)],
            )?;
        }
    }

    // 3. Merge develop into the current branch
    run_command("git", &["merge", "develop"])?;
    println!("Merged 'develop' into the current branch.");

    // 4. Run pnpm clean and install
    run_command("pnpm", &["clean"])?;
    run_command("pnpm", &["install", "--no-frozen-lockfile"])?;
    println!("Ran 'pnpm clean' and 'pnpm install --no-frozen-lockfile' successfully.");

    Ok(())
}