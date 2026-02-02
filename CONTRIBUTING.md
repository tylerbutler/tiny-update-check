# Contributing

Thank you for your interest in contributing!

## Development Setup

1. Clone the repository
2. Install Rust via [rustup](https://rustup.rs/)
3. Install [just](https://github.com/casey/just) for task running
4. Run `just ci` to verify your setup

## Making Changes

1. Create a feature branch from `main`
2. Make your changes
3. Run `just ci` to ensure all checks pass
4. Commit using [conventional commits](https://www.conventionalcommits.org/)
5. Open a pull request

## Commit Message Format

This project uses conventional commits. Each commit message should follow this format:

```
type(scope): description

[optional body]

[optional footer]
```

### Types

- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation only changes
- `style`: Changes that do not affect the meaning of the code
- `refactor`: A code change that neither fixes a bug nor adds a feature
- `perf`: A code change that improves performance
- `test`: Adding missing tests or correcting existing tests
- `build`: Changes that affect the build system or external dependencies
- `ci`: Changes to CI configuration files and scripts
- `chore`: Other changes that don't modify src or test files

### Examples

```
feat: add user authentication
fix(parser): handle empty input correctly
docs: update API documentation
```

## Pull Request Process

1. Ensure your PR title follows the conventional commit format
2. Update documentation if needed
3. Add tests for new functionality
4. Ensure all CI checks pass
5. Request review from maintainers

## Code Style

- Run `just format` before committing
- Run `just lint` to check for issues
- Follow existing patterns in the codebase

## License

By contributing, you agree that your contributions will be licensed under the same license as the project.
