
# Installation Guide

Welcome to _**MetaSSR**_! This guide will walk you through the installation process so you can start building with the framework in no time.

## Table of Contents


- [Manual Installation Steps from Source](#manual-installation-steps-from-source)
  - [Prerequisites](#prerequisites)
  - [1. Clone Git Repository](#1-clone-git-repository)
  - [2. Compiling](#2-compiling)
  - [3. Add the CLI Binary to PATH (Linux)](#3-add-the-cli-binary-to-path-linux)
  - [4. Create Your First Project](#4-create-your-first-project)
- [Conclusion](#conclusion)

## Manual Installation Steps from Source

### Prerequisites

Before installing _**MetaSSR**_, ensure you have the following installed on your machine:

- **Git**: v2.25.0 or higher (optional but recommended)
- **Metacall**: v0.8.1 or higher
- **Rust**: v1.76 (optional but recommended)

You can verify the installation of these tools using the following commands:

```bash
rustc -v
git --version
```

### 1. Clone Git Repository

At first, you need to clone this repository:

```bash
$ git clone https://github.com/metacall/rust-http-ssr.git metassr
$ cd metassr
```

### 2. Compiling

After cloning the repo and getting inside it, compile it via `cargo`, the package manager of Rust programming languages:

```bash
$ cargo build --release
```

### 3. Add the CLI Binary to PATH (Linux)

Now, you'll want to make the binary of `metassr-cli` globally accessible. To do this on Linux, add the binary to your PATH:

```bash
sudo ln -s $(pwd)/target/release/metassr-cli /usr/local/bin/metassr-cli
```

### 4. Create Your First Project

After completing the above steps, you'll be able to create your first web application with ***MetaSSR***!

```bash
$ metassr-cli create <project-name>
```

## Conclusion

You have successfully installed and set up your first SSR framework project! Explore the [docs](../README.md) for more advanced features and customization options.

If you encounter any issues during installation, please reach out to our community on [GitHub](https://github.com/metacall/rust-http-ssr) and open a new issue!

Happy coding!
