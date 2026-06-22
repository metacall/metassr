
# Installation Guide

Welcome to _**MetaSSR**_! This guide will walk you through the installation process so you can start building with the framework in no time.

## Table of Contents


- [Installation Guide](#installation-guide)
  - [Table of Contents](#table-of-contents)
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
$ git clone https://github.com/metacall/metassr.git metassr
$ cd metassr
```

### 2. Compiling

After cloning the repo and getting inside it, compile it via `cargo`, the package manager of Rust programming languages:

```bash
$ cargo build --release
```

### 3. Add the CLI Binary to PATH (Linux)

Now, you'll want to make the binary of `metassr` globally accessible. To do this on Linux, add the binary to your PATH:

```bash
sudo ln -s $(pwd)/target/release/metassr /usr/local/bin/metassr
```

### 4. Create Your First Project

After completing the above steps, you'll be able to create your first web application with ***MetaSSR***!

```bash
$ metassr create <project-name>
```

## Deployment

### Prerequisites
- **Docker Installed**

### Step-by-Step Deployment

#### 1. Create & Build Your App

Scaffold a new MetaSSR app and bundle its assets:

```sh
metassr create ${name of the app}
cd ${name of the app}
npm install
npm run build:ssr
```

Example:-
```sh
metassr create vis
cd vis
npm install
npm run build:ssr
```

#### 2. Build the Docker Image

Build a production-ready Docker image using the release stage:

```sh
docker build --build-arg APP_NAME=${name of app} -t metassr-debug .
```

Example:
```sh
docker build --build-arg APP_NAME=vis -t metassr-debug .
```

#### 3. Test Locally

Run the container and verify everything works before deploying:

Example:
```sh
 docker run -p 8080:8080 metassr-debug
```

Visit [http://localhost:8080](http://localhost:8080) and check logs with:

```sh
docker logs <container-id>
```

#### 5. Verify & Monitor

- **Endpoint**: `GET /` should return server-side rendered content.
- **Logging**: Set `RUST_LOG=info` in your environment for detailed logs.
- **Scaling**: Add replicas via Docker Compose's `scale` option or a Kubernetes deployment.


## Conclusion

You have successfully installed and set up your first SSR framework project! Explore the [docs](../README.md) for more advanced features and customization options.

If you encounter any issues during installation, please reach out to our community on [GitHub](https://github.com/metacall/metassr) and open a new issue!

Happy coding!
