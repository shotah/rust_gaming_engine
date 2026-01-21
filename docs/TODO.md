# Voxel Forge - Development Roadmap

> A phased approach to building the game engine. Check off items as completed!

---

## Phase 1: Foundation 🏗️
*Get something on screen and establish core architecture*

### Window & Rendering Bootstrap
- [ ] Create window with winit
- [ ] Initialize wgpu device and surface
- [ ] Set up render loop with clear color
- [ ] Add basic FPS counter / debug overlay
- [ ] Handle window resize events

### Camera System
- [ ] Implement first-person camera
- [ ] Mouse look (pitch/yaw)
- [ ] WASD movement
- [ ] Sprint and crouch modifiers
- [ ] Camera configuration (FOV, sensitivity)

### Input System
- [ ] Keyboard input handling
- [ ] Mouse input handling  
- [ ] Input mapping system (rebindable keys)
- [ ] Input state queries (is_pressed, just_pressed, just_released)

---

## Phase 2: Voxel World 🌍
*The heart of a Minecraft-like game*

### Block System
- [ ] Block enum/registry
- [ ] Block properties (solid, transparent, light emission)
- [ ] Block textures mapping
- [ ] Block state system (e.g., door open/closed)

### Chunk Data Structure
- [ ] Chunk struct (16x16x256 or 16x16x16 sections)
- [ ] Efficient block storage (palette compression?)
- [ ] Chunk coordinate system
- [ ] Neighbor chunk references

### Chunk Meshing
- [ ] Basic cube mesh generation
- [ ] Face culling (don't render hidden faces)
- [ ] Greedy meshing optimization
- [ ] Ambient occlusion calculation
- [ ] Texture UV mapping
- [ ] Mesh rebuilding on block changes

### Chunk Management
- [ ] Chunk loading/unloading based on player position
- [ ] Chunk render distance configuration
- [ ] Chunk priority queue (closest first)
- [ ] Background chunk meshing (async)

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
- [ ] Ray casting for block selection
- [ ] Block breaking
- [ ] Block placing
- [ ] Block highlight rendering

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
- [ ] Settings menu
- [ ] World selection

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

## Current Focus 🎯

**Start with Phase 1!**

Recommended first tasks:
1. [ ] Window creation with winit
2. [ ] wgpu initialization
3. [ ] Clear screen with a color
4. [ ] Basic render loop

Once you can see a colored window, you're off to the races! 🚀

---

## Notes

- Each phase builds on the previous
- Items can be done in parallel within a phase
- Some items marked "(Later)" can be deferred
- Check items off as you complete them!
- Add new items as you discover needs

*Last updated: January 2026*
