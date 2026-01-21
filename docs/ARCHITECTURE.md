# Voxel Forge - Architecture Overview

This document provides a high-level overview of the Voxel Forge game engine architecture.

## System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Application Layer                         │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌────────────┐ │
│  │   Game UI   │ │   Input     │ │   Audio     │ │  Scripting │ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                         Engine Layer                             │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌────────────┐ │
│  │  Rendering  │ │    World    │ │   Physics   │ │ Networking │ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                          Core Layer                              │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌────────────┐ │
│  │     ECS     │ │  Resources  │ │   Events    │ │   Utils    │ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                        Platform Layer                            │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌────────────┐ │
│  │    wgpu     │ │    winit    │ │    tokio    │ │   rodio    │ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## Core Systems

### 1. Entity Component System (ECS)

We use `bevy_ecs` for our entity component system, providing:
- High-performance entity management
- Cache-friendly data layout
- Parallel system execution
- Component-based architecture

```rust
// Example entity with components
world.spawn((
    Position { x: 0.0, y: 64.0, z: 0.0 },
    Velocity { x: 0.0, y: 0.0, z: 0.0 },
    Player { name: "Steve".to_string() },
));
```

### 2. Rendering System

Built on `wgpu` for cross-platform GPU access:

- **Chunk Renderer**: Efficient voxel mesh rendering
- **Entity Renderer**: Animated entity rendering
- **UI Renderer**: Immediate-mode UI overlay
- **Post-Processing**: Effects pipeline

### 3. World System

Manages the voxel world:

- **Chunk Manager**: Loading/unloading chunks
- **World Generator**: Procedural terrain generation
- **Block Registry**: Block type definitions
- **Persistence**: Save/load world data

### 4. Physics System

Handles all physics simulation:

- **Collision Detection**: AABB and voxel collisions
- **Movement**: Player and entity movement
- **Fluids**: Water and lava simulation

### 5. Networking System

Client-server architecture using QUIC:

- **Protocol**: Custom binary protocol
- **State Sync**: Delta compression
- **Chunk Streaming**: Progressive chunk loading

## Data Flow

```
┌──────────────────────────────────────────────────────────────┐
│                         Game Loop                             │
│                                                               │
│  ┌─────────┐   ┌─────────┐   ┌─────────┐   ┌─────────┐      │
│  │  Input  │ → │ Update  │ → │ Physics │ → │ Render  │      │
│  └─────────┘   └─────────┘   └─────────┘   └─────────┘      │
│       ↑                                          │           │
│       └──────────────────────────────────────────┘           │
│                         (60 FPS)                              │
└──────────────────────────────────────────────────────────────┘
```

## Module Structure

```
src/
├── lib.rs                 # Library root, public API
├── main.rs                # Application entry point
│
├── engine/                # Core engine functionality
│   ├── mod.rs
│   ├── app.rs             # Application lifecycle
│   ├── config.rs          # Configuration management
│   └── time.rs            # Time and delta tracking
│
├── rendering/             # Graphics subsystem
│   ├── mod.rs
│   ├── renderer.rs        # Main renderer
│   ├── pipeline.rs        # Render pipelines
│   ├── mesh.rs            # Mesh generation
│   ├── texture.rs         # Texture management
│   └── shader.rs          # Shader compilation
│
├── world/                 # World management
│   ├── mod.rs
│   ├── chunk.rs           # Chunk data structure
│   ├── block.rs           # Block definitions
│   ├── generator.rs       # World generation
│   └── persistence.rs     # Save/load
│
├── physics/               # Physics engine
│   ├── mod.rs
│   ├── collision.rs       # Collision detection
│   ├── movement.rs        # Movement physics
│   └── fluid.rs           # Fluid simulation
│
├── ecs/                   # ECS extensions
│   ├── mod.rs
│   ├── components.rs      # Common components
│   └── systems.rs         # Common systems
│
├── networking/            # Multiplayer
│   ├── mod.rs
│   ├── server.rs          # Server implementation
│   ├── client.rs          # Client implementation
│   └── protocol.rs        # Network protocol
│
├── audio/                 # Audio system
│   ├── mod.rs
│   └── player.rs          # Audio playback
│
└── ui/                    # User interface
    ├── mod.rs
    └── widgets.rs         # UI components
```

## Performance Considerations

### Memory Layout
- Chunks use `Box<[Block; CHUNK_SIZE]>` for cache efficiency
- ECS components are stored in contiguous arrays
- Resource handles avoid unnecessary cloning

### Parallelism
- Chunk generation runs on `rayon` thread pool
- ECS systems execute in parallel where possible
- Rendering uses multi-threaded command encoding

### Optimization Targets
- **60 FPS** minimum on target hardware
- **16 chunks** render distance minimum
- **<100ms** chunk generation time
- **<50MB** base memory footprint

## Future Considerations

- WebGPU support for browser deployment
- VR/AR rendering paths
- Advanced LOD system
- Infinite vertical worlds
