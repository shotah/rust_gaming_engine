# Voxel Forge - Development Roadmap

> A phased approach to building the game engine. Check off items as completed!

---

## Phase 1: Foundation 🏗️
*Get something on screen and establish core architecture*

### Window & Rendering Bootstrap
- [x] Create window with winit
- [x] Initialize wgpu device and surface
- [x] Set up render loop with clear color
- [x] Add basic FPS counter / debug overlay
- [x] Handle window resize events

### Camera System
- [x] Implement first-person camera
- [x] Mouse look (pitch/yaw)
- [x] WASD movement
- [x] Sprint and crouch modifiers
- [x] Camera configuration (FOV, sensitivity)

### Input System
- [x] Keyboard input handling
- [x] Mouse input handling  
- [ ] Input mapping system (rebindable keys)
- [x] Input state queries (is_pressed, just_pressed, just_released)

---

## Phase 2: Voxel World 🌍
*The heart of a Minecraft-like game*

### Block System
- [x] Block enum/registry
- [x] Block properties (solid, transparent, light emission)
- [x] Block textures mapping (procedural texture atlas)
- [ ] Per-face textures (logs: bark on sides, rings on top/bottom)
- [ ] Block state system (e.g., door open/closed)

### Chunk Data Structure
- [x] Chunk struct (16x16x256 or 16x16x16 sections)
- [ ] Efficient block storage (palette compression?)
- [x] Chunk coordinate system
- [ ] Neighbor chunk references

### Chunk Meshing
- [x] Basic cube mesh generation
- [x] Face culling (don't render hidden faces)
- [x] Greedy meshing optimization (80-90% triangle reduction!)
- [ ] Ambient occlusion calculation
- [x] Texture UV mapping
- [ ] Mesh rebuilding on block changes

### Chunk Management
- [x] Chunk loading/unloading based on player position
- [x] Chunk render distance configuration
- [x] Frustum culling (don't render chunks outside view)
- [ ] Chunk priority queue (closest first)
- [x] Parallel chunk meshing with rayon
- [x] Background chunk generation (threaded worker)

### Performance Foundations ⚡
- [x] Greedy meshing algorithm (merges adjacent faces → 80-90% fewer triangles)
- [x] Frustum culling (skip chunks outside camera view)
- [x] Parallel chunk meshing with rayon (near-linear scaling)
- [x] Benchmarks with criterion (single chunk: ~0.5ms, 9 chunks parallel: ~1.8ms)

---

## Phase 3: World Generation 🌲
*Procedural terrain that's fun to explore*

### Terrain Generation
- [ ] 2D heightmap noise (simplex/perlin)
- [ ] 3D density noise for overhangs/caves
- [ ] Multiple noise octaves
- [ ] Terrain height scaling

### Biomes
- [ ] Temperature/humidity noise maps
- [ ] Biome definitions (plains, forest, desert, etc.)
- [ ] Biome-specific terrain shapes
- [ ] Smooth biome transitions

### Features & Decoration
- [ ] Tree generation (multiple types)
- [ ] Grass and flower placement
- [ ] Ore distribution
- [ ] Cave carving
- [ ] Water/lake generation

### Structures (Later)
- [ ] Structure templates
- [ ] Village generation
- [ ] Dungeon generation
- [ ] Stronghold generation

---

## Phase 4: Physics & Collision 💥
*Make the world feel solid*

### Collision Detection
- [ ] AABB collision primitives
- [ ] Player bounding box
- [ ] Block collision boxes
- [ ] Swept AABB for continuous collision

### Player Physics
- [ ] Gravity
- [ ] Ground detection
- [ ] Jumping
- [ ] Walking up slabs/stairs
- [ ] Swimming/floating in water
- [ ] Fall damage calculation

### Block Interaction
- [x] Ray casting for block selection
- [x] Block breaking
- [x] Block placing
- [ ] Block highlight rendering (wireframe on targeted block)
- [ ] Crosshair HUD element

---

## Phase 5: Lighting 💡
*Make it beautiful*

### Sky Lighting
- [ ] Skylight propagation (from top down)
- [ ] Day/night cycle
- [ ] Sun/moon rendering
- [ ] Sky gradient colors

### Block Lighting
- [ ] Light-emitting blocks (torches, glowstone)
- [ ] Light propagation algorithm (BFS)
- [ ] Light level storage per block

### Rendering
- [ ] Per-vertex lighting
- [ ] Smooth lighting interpolation
- [ ] Shadow mapping (optional, advanced)
- [ ] Ambient occlusion

---

## Phase 6: Entities & ECS 🧟
*Things that move and live*

### ECS Setup
- [ ] Integrate bevy_ecs properly
- [ ] Define core components (Position, Velocity, Health, etc.)
- [ ] System scheduling
- [ ] Event system

### Player Entity
- [ ] Player component
- [ ] Inventory component
- [ ] Hotbar selection
- [ ] Player rendering (third-person view)

### Mobs (Later)
- [ ] Basic mob spawning
- [ ] Mob AI pathfinding
- [ ] Mob rendering
- [ ] Mob interactions

---

## Phase 7: User Interface 🖥️
*Menus and HUD*

### In-Game HUD
- [ ] Crosshair
- [ ] Hotbar
- [ ] Health/hunger bars
- [ ] Debug screen (F3)

### Menus
- [ ] Main menu
- [ ] Pause menu
- [ ] Settings menu (see Settings System below)
- [ ] World selection

### Settings System
*Core settings infrastructure - can be implemented before full UI*

#### Settings Data Structure
- [ ] GameSettings struct with all configurable options
- [ ] Settings categories (Video, Audio, Controls, Gameplay)
- [ ] Default values and validation ranges
- [ ] Settings change events/callbacks

#### Video Settings
- [ ] Resolution selection
- [ ] Fullscreen / Windowed / Borderless
- [ ] VSync toggle
- [ ] Render distance (chunks)
- [ ] FOV slider (60-120°)
- [ ] Graphics quality presets (Low/Medium/High/Ultra)
- [ ] Max FPS limiter

#### Audio Settings  
- [ ] Master volume
- [ ] Music volume
- [ ] Sound effects volume
- [ ] Ambient volume
- [ ] Mute toggle

#### Control Settings
- [ ] Mouse sensitivity
- [ ] Invert Y-axis toggle
- [ ] Key rebinding system
- [ ] Controller support / deadzone settings

#### Gameplay Settings
- [ ] Difficulty selection
- [ ] GUI scale
- [ ] Language selection
- [ ] Chat visibility
- [ ] Show coordinates toggle

#### Settings Persistence
- [ ] Serialize settings to JSON/TOML
- [ ] Settings file location (~/.config/voxel-forge/ or AppData)
- [ ] Load settings on startup
- [ ] Save settings on change
- [ ] Reset to defaults option

#### Settings UI
- [ ] Tabbed settings panel
- [ ] Sliders, toggles, dropdowns components
- [ ] Apply / Cancel / Reset buttons
- [ ] Live preview for some settings (FOV, sensitivity)
- [ ] Restart required indicator for some settings

### Inventory
- [ ] Inventory grid rendering
- [ ] Item drag and drop
- [ ] Crafting grid
- [ ] Crafting recipes

---

## Phase 8: Audio 🔊
*Bring the world to life*

### Audio System
- [ ] Audio engine initialization (rodio)
- [ ] 3D positional audio
- [ ] Volume controls
- [ ] Audio categories (music, effects, ambient)

### Sound Effects
- [ ] Block breaking/placing sounds
- [ ] Footstep sounds
- [ ] Ambient sounds
- [ ] UI sounds

### Music
- [ ] Background music playback
- [ ] Music track selection
- [ ] Crossfading between tracks

---

## Phase 9: Persistence 💾
*Save and load worlds*

### World Saving
- [ ] Chunk serialization format (bincode?)
- [ ] Region file system (like Minecraft's)
- [ ] Auto-save system
- [ ] Save on exit

### World Loading
- [ ] Load chunks from disk
- [ ] Handle missing chunks (generate new)
- [ ] World metadata (seed, spawn point, etc.)

### Player Data
- [ ] Save player position
- [ ] Save inventory
- [ ] Save game settings

---

## Phase 10: Networking 🌐
*Multiplayer support*

### Server
- [ ] Dedicated server mode
- [ ] Client connection handling
- [ ] Player authentication
- [ ] Server tick loop

### Protocol
- [ ] Packet definitions
- [ ] Chunk streaming
- [ ] Entity synchronization
- [ ] Chat messages

### Client
- [ ] Server connection
- [ ] Client-side prediction
- [ ] Lag compensation
- [ ] Interpolation

---

## Phase 11: Polish & Extras ✨
*Make it shine*

### Graphics Enhancements
- [ ] Post-processing (bloom, SSAO)
- [ ] Water rendering (reflections, waves)
- [ ] Particle systems
- [ ] Weather effects (rain, snow)
- [ ] Clouds

### Performance
- [ ] Frustum culling
- [ ] Occlusion culling
- [ ] LOD for distant chunks
- [ ] Profiling and optimization

### Modding (Future)
- [ ] Plugin/mod API design
- [ ] Scripting support (Lua? WASM?)
- [ ] Resource pack loading

---

## Phase 12: AI Model Creation System 🤖
*Enable Cursor AI to create and validate 3D art for blocks AND entities*

### Block Models (Voxel-based)
- [ ] Define voxel model schema (JSON) for non-standard blocks
- [ ] Support for palette-based coloring
- [ ] Model metadata (name, size, origin, collision bounds)
- [ ] Reuse chunk meshing for voxel→mesh conversion

### Entity Models (Polygon-based, Minecraft-style)
- [ ] Hierarchical box model format (cuboids with transforms)
- [ ] Entity schema: parts (head, body, limbs), joints, pivot points
- [ ] Texture UV mapping per face
- [ ] Skeleton/rig definition for animation
- [ ] Example: Player = head(8x8x8) + body(8x12x4) + arms + legs

### Advanced Entity Options (Future)
- [ ] High-res voxel sculpting → marching cubes → smooth mesh
- [ ] Simple glTF import for external models
- [ ] Procedural mesh generation (spheres, cylinders, capsules)

### Model Loading & Rendering
- [ ] Block model parser/loader
- [ ] Entity model parser/loader  
- [ ] Model preview renderer (orbit camera, centered)
- [ ] Skeletal animation playback

### Offscreen Rendering Pipeline
- [ ] wgpu offscreen render target (render to texture)
- [ ] PNG export from render target
- [ ] CLI: `voxel-forge render-model <model.json> -o preview.png`
- [ ] CLI: `voxel-forge render-entity <entity.json> --pose idle -o preview.png`
- [ ] Multiple angle renders (front, side, isometric, turntable)

### AI Workflow Integration  
- [ ] AI generates JSON → engine renders → AI reads PNG to validate
- [ ] Batch rendering for rapid iteration
- [ ] Diff visualization (compare renders)
- [ ] Template primitives (cube, sphere, cylinder, capsule)

### Model Library
- [ ] Block models: doors, fences, stairs, slabs, torches
- [ ] Entity models: player, zombie, skeleton, creeper-style
- [ ] Item models: tools, weapons, food (flat sprites or 3D)
- [ ] Props: furniture, vegetation, decorations
- [ ] Animation sets: idle, walk, attack, death

---

## Current Focus 🎯

**Phase 1 Foundation: ✅ COMPLETE!**
**Phase 2 Voxel World: ✅ CORE COMPLETE!**

Rendering **113 chunks** at ~58 FPS with:
- ✅ Block registry with 18 block types
- ✅ Chunk data structure (16x16x16 sections)
- ✅ Face-culled + greedy mesh generation
- ✅ WGSL shader with directional lighting
- ✅ Depth buffering
- ✅ Procedural terrain with trees
- ✅ Dynamic chunk loading/unloading
- ✅ Background threaded generation

**Next priorities:**
1. [x] Greedy meshing optimization ✅
2. [x] Chunk loading/unloading based on player position ✅
3. [x] Texture mapping for blocks ✅
4. [x] Block breaking/placing ✅
5. [ ] Block selection highlight ← **UP NEXT**

---

## Notes

- Each phase builds on the previous
- Items can be done in parallel within a phase
- Some items marked "(Later)" can be deferred
- Check items off as you complete them!
- Add new items as you discover needs

*Last updated: January 2026*
