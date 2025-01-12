# 🔄 PR Manager Tool

> A Rust implementation of [pr-conflict-resolver](https://github.com/xR0am/pr-conflict-resolver), designed to efficiently manage and repost GitHub Pull Requests.

## ✨ Features

- 🔄 Clones PR repositories (including forks)
- 🔀 Automatically merges base branch
- 📝 Creates a new PR with reference to the original
- 🏷️ Maintains PR title, description, and attribution

## 📋 Prerequisites

Before using this tool, ensure you have:

- ⚙️ Rust installed on your system
- 🌿 Git installed and configured
- 🔑 A GitHub personal access token with appropriate permissions
- 📂 Access to the target repository

## 🔧 Environment Variables

Set the following environment variables:

```bash
GITHUB_TOKEN=your_github_personal_access_token
REPO_OWNER=owner_username
REPO_NAME=repository_name
BASE_BRANCH=target_branch  # Optional, defaults to "develop"
```

## 📥 Installation

1. Clone this repository:
   ```bash
   git clone https://github.com/your-username/pr-manager.git
   cd pr-manager
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

## 🚀 Usage

### Basic Command
```bash
./pr-manager <pr_number> [--no-interactive]
```

### Arguments
| Argument | Description |
|----------|-------------|
| `pr_number` | The number of the PR you want to repost |
| `--no-interactive` | (Optional) Run without interactive prompts |

### Example
```bash
./pr-manager 123
```

## 🔄 Process Flow

1. Clones the repository containing the PR
2. Creates a new branch named `pr{number}_fix`
3. Merges the base branch into the new branch
4. Creates a new PR with:
   - Original title prefixed with "[Repost]"
   - Original description
   - Reference to original PR and author

## ⚠️ Error Handling

- 🔄 Merge conflicts will trigger a notification requiring manual intervention
- 🌐 Network and permission errors are reported with descriptive messages

## 🤝 Contributing

Contributions are welcome! Feel free to:

- 🐛 Report bugs
- 💡 Suggest features
- 🔧 Submit pull requests

## 📄 License

[Add your chosen license here]

---

*This project is a Rust implementation of [pr-conflict-resolver](https://github.com/xR0am/pr-conflict-resolver)*
