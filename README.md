# 🔄 PR Conflict Resolver

A simple Rust tool to help manage Git pull requests and resolve merge conflicts with the develop branch.

## ✨ Features

- 🔄 Automatically checks out PR branches
- 🔀 Merges develop branch automatically
- 📦 Handles package management (pnpm)
- 🛠️ Creates fork branches when needed
- 🔧 Configurable with command-line options

## 📋 Prerequisites

Before using this tool, ensure you have:

- ⚙️ Rust installed on your system
- 🌿 Git installed and configured
- 📦 pnpm installed
- 📂 Access to the target repository

## 🔧 Installation

1. Clone this repository:
   ```bash
   git clone https://github.com/your-username/pr-conflict-resolver.git
   cd pr-conflict-resolver
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

## 🚀 Usage

### Basic Command
```bash
pr-resolver <pr_number> [--no-interactive] [--work-dir <path>]
```

### Arguments
| Argument | Description | Required |
|----------|-------------|----------|
| `pr_number` | The number of the PR you want to work with | Yes |
| `--no-interactive` | Run without interactive prompts | No |
| `--work-dir` | Specify a custom working directory | No |

### Examples
```bash
# Basic usage
pr-resolver 123

# Non-interactive mode
pr-resolver 123 --no-interactive

# With custom working directory
pr-resolver 123 --work-dir /path/to/work/dir
```

## 🔄 Process Flow

1. Attempts to check out the PR branch directly (`pull/{number}/head`)
2. If checkout fails:
   - Creates a new branch named `pr{number}_fork`
   - Fetches the PR branch
   - Checks out the new fork branch
3. Merges the develop branch into the current branch
4. Runs `pnpm clean` and `pnpm install --no-frozen-lockfile`

## ⚠️ Error Handling

The tool handles various scenarios:

- 🔄 Git command failures
- 📦 Package management errors
- ⌨️ Invalid command-line arguments
- 🔧 Missing required parameters

## 🤝 Contributing

Contributions are welcome! Feel free to:

- 🐛 Report bugs
- 💡 Suggest features
- 🔧 Submit pull requests

## 📄 License

MIT

---

*A tool for managing pull requests and resolving merge conflicts*
