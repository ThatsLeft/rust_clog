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

Particles are rendered and updated automatically just add and create particleSystems in game.

Physics and collision handled and updated by the engine, add physics bodies to the physics world in game. Game still have to render the object themselves.


## Features
### Window
- [x] Window open
- [x] Draw background
- [x] Change background
- [x] Draw geometry
- [x] Window resize handling
- [ ] Fullscreen toggle
- [ ] VSync control
- [ ] Custom app icon

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

### Physics 
- [ ] Point-in-rectangle detection
- [ ] Collision response (bounce, slide)
- [ ] Collision layers/masks
- [ ] Trigger zones (non-physical collisions)
- [ ] Continuous collision detection for fast objects
- [ ] Spatial partitioning for broad-phase collision
- [ ] Friction implementation in collision response

#### Core Physics
- [x] RigidBody component (position, velocity, acceleration, mass)
- [x] Body types: Static, Dynamic, Kinematic
- [x] Physics materials (restitution, friction, drag)
- [x] Force accumulation and integration
- [x] Body sleeping/activation system 
- [x] Physics substeps for accuracy 

#### Gravity
- [x] Global uniform gravity
- [x] Individual gravity fields per body
- [x] Multiple falloff types (Linear, InverseSquare, Constant, Custom)
- [x] Configurable gravity direction and strength
- [x] Orbital gravity with center points


#### Collision System
- [x] AABB-AABB collision detection
- [x] Circle-Circle collision detection
- [x] AABB-Circle collision detection
- [x] Contact point calculation
- [x] Impulse-based collision response
- [x] Position correction to prevent sinking
- [x] Penetration depth calculation

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
- [x] Pause/resume functionality

### UI System
- [ ] Basic UI elements (text, buttons, panels)
- [ ] UI layout system
- [ ] UI event handling
- [ ] Debug UI/console
- [ ] Text Rendering

### Utilities
- [ ] Math utilities (vectors, matrices, interpolation)
- [ ] Random number generation
- [ ] Configuration file loading
- [ ] Logging system
- [ ] Debug utilities
- [ ] Physics debug visualization
- [ ] Performance profiling tools
- [ ] Collision statistics
- [ ] Memory usage optimization
- [ ] Multi-threading support
