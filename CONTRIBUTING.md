# Contributing to MetaSSR

Thank you for your interest in contributing to MetaSSR! We welcome contributions from the community to help improve and expand the framework. Please follow the guidelines below to ensure your contributions are effective and align with the project's goals.

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
   git commit -m "Add feature: Description of the feature"
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
cargo test
```

also, you can test one of web applications that located at [tests](../../tests/) directory.

**Example:**
```bash
$ cargo run --bin metassr-cli -- --root=tests/web-app --debug-mode=all run 
```


### 7. Code of Conduct

Please adhere to our [Code of Conduct](code-of-conduct.md) while participating in the MetaSSR community.

## Getting Help

If you have any questions or need assistance, feel free to reach out to us through the project's [discussion forum](https://github.com/metacall/metassr/discussions) or open an issue.

Thank you for contributing to MetaSSR!

