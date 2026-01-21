# Contributing to Voxel Forge

Thank you for your interest in contributing to Voxel Forge! This document provides guidelines and instructions for contributing.

## Code of Conduct

Be respectful, inclusive, and constructive. We're all here to build something amazing together.

## Getting Started

### Prerequisites

1. **Rust**: Install the latest stable version via [rustup](https://rustup.rs/)
2. **Git**: For version control
3. **Make**: For running build commands

### Setup

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/voxel-forge.git
cd voxel-forge

# Add upstream remote
git remote add upstream https://github.com/ORIGINAL_OWNER/voxel-forge.git

# Setup development environment
make setup
```

## Development Workflow

### 1. Create a Branch

```bash
# Update main branch
git checkout main
git pull upstream main

# Create feature branch
git checkout -b feature/your-feature-name
```

### 2. Make Changes

- Write clean, documented code
- Follow the existing code style
- Add tests for new functionality
- Update documentation as needed

### 3. Test Your Changes

```bash
# Run all checks
make check

# This runs:
# - make fmt-check  (formatting)
# - make lint       (clippy)
# - make test       (tests)
```

### 4. Commit Your Changes

```bash
# Stage changes
git add .

# Commit (pre-commit hooks will run automatically)
git commit -m "feat: add amazing feature"
```

#### Commit Message Format

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `style`: Code style (formatting, etc.)
- `refactor`: Code refactoring
- `perf`: Performance improvement
- `test`: Adding tests
- `chore`: Maintenance tasks

**Examples:**
```
feat(rendering): add shadow mapping
fix(physics): correct AABB collision detection
docs(readme): update installation instructions
perf(world): optimize chunk meshing
```

### 5. Push and Create PR

```bash
# Push to your fork
git push origin feature/your-feature-name
```

Then open a Pull Request on GitHub.

## Code Style

### Rust Guidelines

- Follow the official [Rust Style Guide](https://doc.rust-lang.org/style-guide/)
- Use `rustfmt` for formatting (configured in `rustfmt.toml`)
- Address all `clippy` warnings
- Write documentation for public APIs

### Documentation

- Use `///` for public item documentation
- Use `//!` for module-level documentation
- Include examples in doc comments where appropriate
- Keep comments concise and meaningful

```rust
/// Generates a chunk at the specified coordinates.
///
/// # Arguments
///
/// * `x` - The chunk X coordinate
/// * `z` - The chunk Z coordinate
///
/// # Returns
///
/// A newly generated chunk with terrain data.
///
/// # Example
///
/// ```
/// let chunk = generate_chunk(0, 0);
/// assert!(!chunk.is_empty());
/// ```
pub fn generate_chunk(x: i32, z: i32) -> Chunk {
    // Implementation
}
```

### Testing

- Write unit tests for all new functionality
- Place unit tests in the same file as the code
- Place integration tests in `/tests`
- Use descriptive test names

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_contains_block_at_valid_position() {
        let chunk = Chunk::new();
        chunk.set_block(0, 0, 0, Block::Stone);
        assert_eq!(chunk.get_block(0, 0, 0), Block::Stone);
    }

    #[test]
    #[should_panic]
    fn chunk_panics_on_invalid_position() {
        let chunk = Chunk::new();
        chunk.get_block(100, 0, 0); // Out of bounds
    }
}
```

## Pull Request Guidelines

### Before Submitting

- [ ] All tests pass (`make test`)
- [ ] Code is formatted (`make fmt`)
- [ ] No clippy warnings (`make lint`)
- [ ] Documentation is updated
- [ ] Commit messages follow conventions

### PR Description

Provide a clear description including:
- What changes were made
- Why the changes were needed
- Any breaking changes
- Related issues

### Review Process

1. Automated CI checks must pass
2. At least one maintainer review required
3. Address review feedback
4. Squash commits if requested

## Reporting Issues

### Bug Reports

Include:
- Rust version (`rustc --version`)
- OS and version
- Steps to reproduce
- Expected vs actual behavior
- Relevant logs or screenshots

### Feature Requests

Include:
- Clear description of the feature
- Use case / motivation
- Proposed implementation (if any)
- Alternatives considered

## Questions?

- Open a GitHub Discussion
- Check existing issues and PRs
- Review the documentation

Thank you for contributing! 🦀
