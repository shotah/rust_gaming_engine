# 🎮 Voxel Forge - A Rust Voxel Game Engine

> A high-performance, Minecraft/Hytale-inspired voxel game engine built in Rust.

[![CI](https://github.com/yourusername/voxel-forge/actions/workflows/ci.yml/badge.svg)](https://github.com/yourusername/voxel-forge/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## 🎯 Vision

Build a modern, performant voxel game engine that leverages Rust's safety guarantees and zero-cost abstractions to deliver a smooth, extensible gaming experience.

## 🚀 Core Engine Features

- **Rendering**: wgpu-based pipeline, chunk meshing, frustum culling, lighting, shadows, post-processing
- **World Generation**: Procedural terrain, biomes, caves, structures, infinite world support
- **Physics & Collision**: AABB collision, voxel physics, fluid dynamics
- **Entity Component System**: High-performance ECS with parallel systems
- **Networking**: Client-server architecture, state sync, lag compensation
- **Audio**: 3D positional audio, music, ambient sounds
- **Input & UI**: Keyboard/mouse/gamepad, customizable bindings, in-game UI
- **Resource Management**: Asset loading, hot-reloading, mod support

📋 **See [Development Roadmap](docs/TODO.md) for detailed progress and task tracking.**

## 📁 Project Structure

```
voxel-forge/
├── src/                    # Main source code
│   ├── lib.rs              # Library root
│   ├── main.rs             # Application entry point
│   ├── engine/             # Core engine systems
│   ├── rendering/          # Graphics and rendering
│   ├── world/              # World generation and management
│   ├── physics/            # Physics and collision
│   ├── ecs/                # Entity Component System
│   ├── networking/         # Multiplayer networking
│   ├── audio/              # Audio system
│   └── ui/                 # User interface
├── docs/                   # Documentation
├── scripts/                # Development scripts
├── assets/                 # Game assets (textures, models, sounds)
├── tests/                  # Integration tests
└── benches/                # Performance benchmarks
```

## 🛠️ Development Setup

### Prerequisites

- Rust (latest stable) - [Install Rust](https://rustup.rs/)
- Make (for build commands)
- Git

### Quick Start

```bash
# Clone the repository
git clone https://github.com/yourusername/voxel-forge.git
cd voxel-forge

# Install git hooks
make install-hooks

# Build the project
make build

# Run tests
make test

# Run the game
make run
```

### Available Commands

| Command | Description |
|---------|-------------|
| `make build` | Build the project in debug mode |
| `make release` | Build the project in release mode |
| `make run` | Run the game in debug mode |
| `make test` | Run all tests |
| `make lint` | Run clippy linter |
| `make fmt` | Format code with rustfmt |
| `make fmt-check` | Check code formatting |
| `make check` | Run all checks (fmt, lint, test) |
| `make clean` | Clean build artifacts |
| `make install-hooks` | Install git pre-commit hooks |
| `make docs` | Generate documentation |
| `make bench` | Run benchmarks |

## 📖 Documentation

All documentation lives in the `/docs` folder:

- [Architecture Overview](docs/ARCHITECTURE.md)
- [Contributing Guide](docs/CONTRIBUTING.md)
- [Engine Systems](docs/SYSTEMS.md)

## 🧪 Testing Strategy

- **Unit Tests**: Located alongside source code
- **Integration Tests**: Located in `/tests`
- **Benchmarks**: Located in `/benches`

All tests must pass before commits (enforced via pre-commit hooks).

## 🔧 Code Quality

We enforce strict code quality through:

- **rustfmt**: Consistent code formatting
- **clippy**: Rust linting with pedantic settings
- **Pre-commit hooks**: Automated checks before each commit
- **CI/CD**: GitHub Actions validates all PRs

## 📦 Dependencies Policy

We follow a "latest packages" policy to avoid technical debt:

- Dependencies are kept up-to-date
- Regular dependency audits via `cargo audit`
- Breaking changes are addressed promptly

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run `make check` to ensure all checks pass
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Inspired by Minecraft and Hytale
- Built with love in Rust 🦀
