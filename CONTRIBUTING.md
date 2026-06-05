# Contributing to MetaSSR

Thank you for your interest in contributing to MetaSSR! We welcome contributions from the community to help improve and expand the framework. Please follow the guidelines below to ensure your contributions are effective and align with the project's goals.

## Table of contents
- [Development Setup](#development-setup)
- [How to Contribute](#how-to-contribute)
## Development Setup

To set up your development environment for MetaSSR, choose one of the following methods based on your preferences and system configuration:

### Nix Flake (Recommended)

The fastest way to get started with a fully configured development environment:

1. **Install Nix** (if not already installed):
   ```bash
   sh <(curl --proto '=https' --tlsv1.2 -L https://nixos.org/nix/install) --daemon
   ```

2. **Enable Nix Flakes**:
   ```bash
   mkdir -p ~/.config/nix
   echo "experimental-features = nix-command flakes" >> ~/.config/nix/nix.conf
   ```

3. **Enter Development Shell**:
   ```bash
   nix develop
   ```

This will automatically set up Rust, MetaCall, and all required dependencies in an isolated environment.

<!--
### Installation Script
Run `./install.sh` to download MetaCall and link it for most distros without conflicts.
-->
<!--
### Docker
1. Build: `docker build -t metacall/metassr:dev -f Dockerfile.dev .`
2. Run: `docker run --rm -it metacall/metassr:dev bash`
-->

### Manual Installation

If you prefer to set up dependencies manually:

1. **Install Rust Toolchain**:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

2. **Install MetaCall Runtime**:
   ```bash
   curl -sL https://raw.githubusercontent.com/metacall/install/master/install.sh | sh
   ```

3. **Clone and Build**:
   ```bash
   git clone https://github.com/metacall/metassr.git
   cd metassr
   cargo build --release
   ```

4. **Verify Installation**:
   ```bash
   cargo test
   ./target/release/metassr --help
   ```

## How to Contribute

### 1. Reporting Issues

If you encounter a bug or have a suggestion for a new feature, please report it using the following steps:

1. **Check Existing Issues**: Before creating a new issue, search the [issue tracker](https://github.com/metacall/metassr/issues) to see if your issue or feature request has already been reported.
2. **Create a New Issue**: If your issue is not listed, create a new issue with a clear and descriptive title. Provide as much detail as possible about the problem or feature request.

### 2. Submitting Code Contributions

To contribute code, follow these steps:

1. **Fork the Repository**: Fork the MetaSSR repository to your own GitHub account. You can do this by clicking the "Fork" button at the top right of the repository page.
2. **Clone Your Fork**: Clone your forked repository to your local machine using the following command:

   ```bash
   git clone https://github.com/yourusername/metassr.git
   ```

3. **Create a Branch**: Create a new branch for your changes. Use a descriptive name for the branch that reflects the nature of your work:

   ```bash
   git checkout -b feature/your-feature-name
   ```

4. **Make Changes**: Implement your changes in the new branch. Ensure that your code adheres to the project's coding standards and guidelines.

5. **Write Tests**: Add or update tests to ensure your changes are covered. We use [testing framework] for our tests, and you can find existing tests in the `tests` directory.

6. **Commit Changes**: Commit your changes with a clear and concise commit message. Follow the [commit message conventions](#commit-message-conventions) for consistency:

   ```bash
   git add .
   git commit -m "feat: Description of the feature"
   ```

7. **Push Changes**: Push your changes to your forked repository:

   ```bash
   git push origin feature/your-feature-name
   ```

8. **Create a Pull Request**: Go to the [pull requests page](https://github.com/metacall/metassr/pulls) of the original repository and create a new pull request. Provide a detailed description of your changes and any relevant information.

### 3. Code Review and Merge

Once your pull request is submitted, it will be reviewed by the project maintainers. They may request changes or provide feedback. Please be responsive to their comments and make any necessary adjustments.

### 4. Coding Standards

- **Code Style**: Follow the coding style and conventions used in the existing codebase. This includes indentation, naming conventions, and code organization.
- **Documentation**: Update or add documentation as needed. Ensure that your code changes are reflected in the project documentation.
- **Windows Path Handling**: Windows paths require special handling when passed to JavaScript/MetaCall:

  1. **Canonicalization**: Always use `dunce::canonicalize()` instead of `std::fs::canonicalize()`. On Windows, `std::fs::canonicalize()` returns paths with the `\\?\` extended-length prefix (e.g., `\\?\C:\path\to\file`), which causes issues when passed to MetaCall/rspack. The `dunce` crate removes this prefix when safe, while being a no-op on other platforms.

  2. **JavaScript String Safety**: When passing paths to JavaScript code (e.g., in templates, MetaCall FFI, or generated JS), always use `metassr_utils::js_path::to_js_path()`. This converts backslashes to forward slashes, preventing escape sequence interpretation in JavaScript strings (e.g., `\t` becoming a TAB character).

  ```rust
  use metassr_utils::js_path::to_js_path;

  // ❌ Don't use:
  let path = std::fs::canonicalize(&path)?;
  let js_import = format!(r#"import Page from "{}""#, path.to_str().unwrap());

  // ✅ Use instead:
  let path = dunce::canonicalize(&path)?;
  let js_import = format!(r#"import Page from "{}""#, to_js_path(&path));
  ```

### 5. Commit Message Conventions

Use clear and descriptive commit messages that follow this format:

- **Type**: A short description of the change (e.g., `feat`, `fix`, `refactor`, `chore`, `doc`).
- **Scope**: A brief description of the affected area (optional).
- **Description**: A concise explanation of the change.

**Examples:**

```
feat(cli): new cool feature in the cli
fix(builder): fix a bug in building operation
```

### 6. Testing

Make sure your changes pass all existing and new tests. Run the tests locally before submitting your pull request:

```bash
cargo test --workspace
```

also, you can test one of web applications that located at [tests](../../tests/) directory.

**Example:**
```bash
$ cargo run --bin metassr -- --root=tests/web-app --debug-mode=all run
```


### 7. Code of Conduct

Please adhere to our [Code of Conduct](code-of-conduct.md) while participating in the MetaSSR community.

## Getting Help

If you have any questions or need assistance, feel free to reach out to us through the project's [discussion forum](https://github.com/metacall/metassr/discussions) or open an issue.

Thank you for contributing to MetaSSR!
