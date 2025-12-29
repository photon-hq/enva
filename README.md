# Enva

**Enva** is a secure environment file sharing solution for GitHub organizations. It enables team members to synchronize `.env` files across repositories while maintaining strict access control through GitHub organization membership verification.

## Overview

Enva automatically syncs environment files (`.env`, `.env.local`, etc.) across team members working on the same repository. When you commit code, your environment files are securely stored on a central server. When teammates pull changes, their environment files are automatically updated to match the latest commit.

## Features

- **Automatic Sync** - Environment files sync automatically via Git hooks
- **Organization Security** - Only GitHub organization members can access shared files
- **Multi-file Support** - Handles all files matching `.env*` pattern
- **Commit-based Versioning** - Environment files are tied to specific Git commits (works great with branches)
- **Zero Configuration** - Works immediately after activation in a repository

## Installation

### Prerequisites

- Rust toolchain (for installation via cargo)
- Git repository
- GitHub personal access token **OR** [GitHub CLI](https://cli.github.com/) (installed and authenticated)

### Install via Cargo

```bash
# Install from crates.io
cargo install enva

# Verify installation
enva --version
```

### Build from Source (Optional)

```bash
# Clone the repository
git clone https://github.com/photon-hq/enva.git
cd enva

# Install the CLI
cargo install --path crates/client

# Verify installation
enva --version
```

## Quick Start

### 1. Authenticate

Login using your GitHub token:

```bash
# Option 1: Using a personal access token
enva login --token ghp_your_token_here

# Option 2: Use GitHub CLI (requires gh to be installed and logged in)
enva login --gh
```

**Note:** The `--gh` option requires the [GitHub CLI](https://cli.github.com/) to be installed and authenticated. If you don't have it, use a personal access token instead.

### 2. Activate in a Repository

Navigate to your Git repository and activate enva:

```bash
cd your-repo
enva active
```

This command:
- Verifies you have access to the repository
- Installs Git hooks (`post-commit`, `post-merge`, `post-checkout`)
- Fetches the latest environment files for your current commit

### 3. Work Normally

After activation, enva works automatically:

```bash
# Make changes to code or .env files
echo "API_KEY=secret" >> .env

# Commit your changes
git commit -am "Update API configuration"
# → post-commit hook automatically syncs .env files to server

# Pull changes from teammates
git pull
# → post-merge hook automatically fetches their .env files
```

## Commands

### `enva login`

Authenticate with GitHub:

```bash
# Using personal access token
enva login --token <TOKEN>

# Using GitHub CLI (requires gh CLI installed and logged in)
enva login --gh
```

The `--gh` flag automatically retrieves your token from the GitHub CLI. You must have [gh](https://cli.github.com/) installed and authenticated first (`gh auth login`).

### `enva active`

Activate enva for the current repository:

```bash
enva active
```

This command:
1. Verifies repository ownership/access
2. Installs Git hooks
3. Fetches environment files for the current commit

## Security

Enva enforces strict access control through GitHub:

1. **GitHub Token Authentication** - All requests require a valid GitHub personal access token
2. **Organization Membership** - Users must be members of the repository's GitHub organization
3. **Repository Permissions** - Users must have at least read permissions on the repository

These checks are performed via the GitHub API on every commit and fetch operation, ensuring only authorized team members can access shared environment files.

## How It Works

1. When you activate enva in a repository, Git hooks are installed automatically
2. After each commit, the `post-commit` hook uploads your `.env*` files to the server
3. When you pull changes or switch branches, hooks automatically download the latest environment files for that specific commit
4. All operations verify your GitHub organization membership before proceeding

### Branch-specific Environments

Enva works perfectly with different branches that have different environment setups:

- Each branch can have its own environment configuration
- When you switch branches (`git checkout`), your `.env` files automatically update to match that branch's configuration
- Development, staging, and production branches can maintain separate environment variables
- Team members automatically get the correct environment for whichever branch they're working on

**Example workflow:**
```bash
# Working on development branch
git checkout development
# → .env files automatically update to development configuration

# Switch to production branch
git checkout production
# → .env files automatically update to production configuration

# Create a new feature branch
git checkout -b feature/new-api
# → Inherits .env files from the parent branch
```

## Self-Host

Enva comes with a default server built-in, so you can start using it immediately. However, if you want to host your own Enva server for your organization, the easiest way is using Nixpacks for automatic deployment.

### Deploy with Nixpacks (Recommended)

Nixpacks automatically detects and builds your Rust application with zero configuration.

```bash
# Clone the repository
git clone https://github.com/yourusername/enva.git
cd enva

# Create .env file for server configuration
echo "PORT=8080" > .env

# Build and run with nixpacks
nixpacks build . --name enva-server
docker run -p 8080:8080 enva-server
```

**Deploy to popular platforms:**

**Railway:**
```bash
# Create .env file first
echo "PORT=8080" > .env

# Connect your repo and deploy (auto-detects with nixpacks)
railway up
```

**Render:**
1. Create a `.env` file in your repository:
   ```bash
   PORT=8080
   ```
2. Connect your GitHub repository
3. Select "Web Service"
4. Render auto-detects Rust and uses nixpacks

**Fly.io:**
```bash
# Create .env file first
echo "PORT=8080" > .env

fly launch
# Automatically detects and configures with nixpacks
fly deploy
```

### Manual Deployment

If you prefer manual setup:

```bash
# Clone the repository
git clone https://github.com/yourusername/enva.git
cd enva

# Create .env file for server configuration
echo "PORT=8080" > .env

# Build the server
cargo build --release -p server

# Run the server
./target/release/server
```

The server will read the `PORT` from the `.env` file.

### Configuration

**Required `.env` file:**
```bash
PORT=8080
```

The server stores environment files in the system's config directory:

- **macOS/Linux**: `~/.config/enva/`
- **Windows**: `%APPDATA%\enva\`

No additional configuration is required - the server creates necessary directories automatically.

### Using Your Self-Hosted Server

To use your self-hosted server instead of the default one, you'll need to build a custom client binary with your server URL:

```bash
# Clone the repository
git clone https://github.com/yourusername/enva.git
cd enva

# Configure your server URL in the client's .env file
echo "BASE_URL=https://your-enva-server.com" > crates/client/.env

# Build and install the custom client
cargo install --path crates/client --force
```

Distribute this custom-built binary to your team members so they connect to your self-hosted server instead of the default one.

## Limitations

- Only supports GitHub repositories
- Environment files must match the `.env*` pattern
- Requires internet connection for GitHub API verification

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Authors

Copyright (c) 2025 Photon AI
