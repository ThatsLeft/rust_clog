# Rusty Clog
Rust Sokol Game engine

## Technologies Used
Rust  
Sokol

## Using the App
Clone the repository then start the app from root using cargo.
  
```bash
...\rusclog>cargo run
```
  
The test game will start running.

## Features
### Window
- [x] Window open
- [x] Draw background
- [x] Change background
- [x] Draw geometry
- [x] Window resize handling
- [ ] Fullscreen toggle
- [ ] VSync control

### Camera
- [x] Camera scene
- [x] Camera zoom
- [x] Camera rotation
- [x] Camera shake
- [ ] Camera boundaries
- [ ] Smooth camera
- [ ] Multiple camera modes
- [ ] Camera viewport/culling

### Texture/Sprite
- [x] Load and render sprite
- [x] Change sprite during 
- [x] Sprite flipping (horizontal/vertical)
- [ ] Sprite batching/instancing
- [ ] Sprite tinting/color modulation
- [ ] Sprite scaling
- [ ] Texture atlas management

### Animation
- [x] Load sprite sheet with animations
- [x] Animation looping modes (once, loop, ping-pong)
- [ ] Animation state machine
- [ ] Animation blending/transitions
- [ ] Animation events/callbacks

### Particle system
- [x] Particle struct
- [x] ParticleSystem struct
- [x] Particle physics update
- [x] Particle Emission logic
- [x] Particle Render integration
- [x] Cleanup
- [ ] Emitter shape
- [ ] Physics forces
- [ ] Size and scale
- [ ] Texture support
- [ ] Particle pooling
- [ ] Performance optimization (GPU compute shaders, instanced rendering)

### Collision
- [x] AABB-AABB collision detection
- [x] Circle-Circle collision detection
- [ ] Point-in-rectangle detection
- [ ] Collision response (bounce, slide)
- [ ] Collision layers/masks
- [ ] Trigger zones (non-physical collisions)

### Input Management
- [x] Keyboard input handling
- [x] Mouse input handling
- [x] Input state management (pressed, held, released)
- [ ] Gamepad support
- [ ] Input mapping/binding system

### Scene Management
- [ ] Scene loading/unloading
- [ ] Scene transitions
- [ ] Entity-Component System (ECS) basics
- [ ] Game object lifecycle management
- [ ] Scene persistence/serialization

### Audio
- [ ] Load and play sound effects
- [ ] Background music playback
- [ ] Volume control (master, sfx, music)
- [ ] Audio streaming for large files
- [ ] 3D positional audio (optional)

### Resource Management
- [ ] Asset loading system
- [ ] Resource caching
- [ ] Hot reloading (development)
- [ ] Memory management for assets

### Game Loop & Timing
- [ ] Fixed timestep game loop
- [ ] Delta time calculation
- [ ] Frame rate limiting
- [ ] Pause/resume functionality

### UI System
- [ ] Basic UI elements (text, buttons, panels)
- [ ] UI layout system
- [ ] UI event handling
- [ ] Debug UI/console

### Utilities
- [ ] Math utilities (vectors, matrices, interpolation)
- [ ] Random number generation
- [ ] Configuration file loading
- [ ] Logging system
- [ ] Debug utilities
