# Voxel Forge - Engine Systems

This document details the major systems within the Voxel Forge engine.

## Table of Contents

1. [Rendering System](#rendering-system)
2. [World System](#world-system)
3. [Physics System](#physics-system)
4. [Entity Component System](#entity-component-system)
5. [Networking System](#networking-system)
6. [Audio System](#audio-system)
7. [Input System](#input-system)

---

## Rendering System

The rendering system uses `wgpu` for cross-platform GPU access, supporting Vulkan, Metal, DX12, and WebGPU.

### Pipeline Overview

```
Input Assembly → Vertex Shader → Rasterization → Fragment Shader → Output
```

### Key Components

#### Chunk Renderer
Renders voxel chunks efficiently using:
- **Greedy Meshing**: Combines adjacent faces into larger quads
- **Face Culling**: Only renders exposed faces
- **Texture Arrays**: Single draw call for all block textures

#### Lighting
- **Ambient Light**: Base illumination
- **Skylight**: Propagates from sky downward
- **Block Light**: Emitted by light-source blocks
- **Smooth Lighting**: Interpolated ambient occlusion

#### Post-Processing
- Bloom
- SSAO (Screen Space Ambient Occlusion)
- Depth of Field
- Tone Mapping

---

## World System

Manages the voxel world, including generation, storage, and modification.

### Chunk System

Chunks are 16x256x16 blocks (configurable):

```rust
pub struct Chunk {
    blocks: Box<[Block; CHUNK_VOLUME]>,
    position: ChunkPos,
    dirty: bool,
    mesh: Option<ChunkMesh>,
}
```

### World Generation

Multi-stage generation pipeline:

1. **Terrain Shape**: 3D noise for base terrain
2. **Biome Selection**: 2D noise for biome distribution
3. **Surface Decoration**: Grass, flowers, trees
4. **Cave Carving**: 3D noise for caves
5. **Structure Generation**: Villages, dungeons
6. **Ore Distribution**: Mineral placement

### Biomes

| Biome | Temperature | Humidity | Features |
|-------|-------------|----------|----------|
| Plains | Medium | Medium | Grass, flowers |
| Forest | Medium | High | Trees, bushes |
| Desert | High | Low | Sand, cacti |
| Tundra | Low | Low | Snow, ice |
| Ocean | Any | Max | Water, coral |

---

## Physics System

Handles collision detection, movement, and physical simulation.

### Collision Detection

#### AABB Collision
Axis-Aligned Bounding Box for entity collision:

```rust
pub struct AABB {
    min: Vec3,
    max: Vec3,
}

impl AABB {
    pub fn intersects(&self, other: &AABB) -> bool {
        self.min.x <= other.max.x && self.max.x >= other.min.x &&
        self.min.y <= other.max.y && self.max.y >= other.min.y &&
        self.min.z <= other.max.z && self.max.z >= other.min.z
    }
}
```

#### Voxel Collision
Swept AABB against voxel grid for precise collision response.

### Movement Physics

- **Gravity**: Constant downward acceleration
- **Friction**: Ground and air friction
- **Drag**: Fluid resistance
- **Jump**: Impulse-based jumping

### Fluid Simulation

Cellular automaton for water/lava:
- Flow direction based on neighbors
- Pressure simulation
- Source blocks

---

## Entity Component System

Using `bevy_ecs` for high-performance ECS.

### Core Components

```rust
// Position in 3D space
#[derive(Component)]
pub struct Position(pub Vec3);

// Velocity for movement
#[derive(Component)]
pub struct Velocity(pub Vec3);

// Health for damageable entities
#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

// Player marker and data
#[derive(Component)]
pub struct Player {
    pub name: String,
    pub uuid: Uuid,
}
```

### Systems

Systems run each frame in defined order:

```rust
// Example system
fn movement_system(
    time: Res<Time>,
    mut query: Query<(&mut Position, &Velocity)>,
) {
    for (mut pos, vel) in query.iter_mut() {
        pos.0 += vel.0 * time.delta_seconds();
    }
}
```

### System Ordering

```
Input Systems
    ↓
Game Logic Systems
    ↓
Physics Systems
    ↓
Rendering Systems
```

---

## Networking System

Client-server architecture using QUIC protocol.

### Protocol Design

Binary protocol with message types:

| Message | Direction | Description |
|---------|-----------|-------------|
| `Connect` | C→S | Initial connection |
| `Disconnect` | C→S | Graceful disconnect |
| `ChunkData` | S→C | Chunk block data |
| `EntitySpawn` | S→C | Entity creation |
| `EntityUpdate` | S→C | Entity state change |
| `PlayerInput` | C→S | Player actions |
| `ChatMessage` | Both | Chat communication |

### State Synchronization

- **Delta Compression**: Only send changes
- **Prediction**: Client-side prediction
- **Reconciliation**: Server-authoritative corrections

### Chunk Streaming

1. Client sends position
2. Server calculates visible chunks
3. Priority queue based on distance
4. Compressed chunk data sent
5. Client decompresses and meshes

---

## Audio System

3D positional audio using `rodio`.

### Sound Categories

- **Music**: Background tracks
- **Ambient**: Environmental sounds
- **Effects**: Player actions, events
- **UI**: Interface feedback

### Spatial Audio

```rust
pub struct AudioSource {
    position: Vec3,
    volume: f32,
    falloff: f32,
    looping: bool,
}
```

- Distance-based attenuation
- Stereo panning
- Doppler effect (optional)

---

## Input System

Handles keyboard, mouse, and gamepad input.

### Input Mapping

```rust
pub enum GameAction {
    MoveForward,
    MoveBackward,
    MoveLeft,
    MoveRight,
    Jump,
    Crouch,
    Attack,
    Use,
    Inventory,
    // ...
}

pub struct InputBindings {
    keyboard: HashMap<KeyCode, GameAction>,
    mouse: HashMap<MouseButton, GameAction>,
    gamepad: HashMap<GamepadButton, GameAction>,
}
```

### Input Processing

1. Raw input events from `winit`
2. Mapped to game actions
3. Aggregated into input state
4. Consumed by game systems

### Mouse Handling

- **Look**: Mouse movement controls camera
- **Sensitivity**: Configurable
- **Raw Input**: Optional for precision
- **Cursor Lock**: Captured during gameplay

---

## System Interaction Diagram

```
┌─────────┐     ┌─────────┐     ┌─────────┐
│  Input  │────→│   ECS   │────→│ Physics │
└─────────┘     └─────────┘     └─────────┘
                     │               │
                     ↓               ↓
              ┌─────────┐     ┌─────────┐
              │  World  │←───→│  Audio  │
              └─────────┘     └─────────┘
                     │
                     ↓
              ┌─────────┐     ┌─────────┐
              │ Render  │←───→│   Net   │
              └─────────┘     └─────────┘
```

Each system is designed to be modular and replaceable, enabling easy testing and future improvements.
