use sokol::{app as sapp, gfx as sg, glue as sglue};
use std::ffi::{self, CString};
use std::collections::HashMap;
use crate::engine::{camera, AnimationManager, Camera2D, Game, GameConfig, InputManager, ParticleSystem, Renderer, SystemState};

pub struct App<T: Game> {
    game: T,
    config: GameConfig,
}

// State structure that will be passed through sokol callbacks
struct AppState<T: Game> {
    game: T,
    pass_action: sg::PassAction,
    renderer: Renderer,
    input: InputManager,
    camera: Camera2D,
    animation_manager: AnimationManager,
    particle_systems: HashMap<String, ParticleSystem>,


    // Engine manages system state
    system_state: SystemState,
    previous_system_state: SystemState,
    system_state_time: f32,
    loading_progress: f32,
    
    // Track background state behavior
    background_frame_throttle: u32,
    background_frame_counter: u32,
}

impl<T: Game> App<T> {
    // default config
    pub fn new(game: T) -> Self {
        let config = T::config();
        Self { game, config }
    }

    pub fn run(self) {

        // Create the state that will be passed to callbacks
        let mut pass_action = sg::PassAction::new();
        pass_action.colors[0] = sg::ColorAttachmentAction {
            load_action: sg::LoadAction::Clear,
            clear_value: self.config.background_color,
            ..Default::default()
        };

        let state = Box::new(AppState {
            game: self.game,
            pass_action,
            renderer: Renderer::new(),
            input: InputManager::new(),
            camera: Camera2D::new(),
            animation_manager: AnimationManager::new(),
            particle_systems: HashMap::new(),
            system_state: SystemState::Starting,
            previous_system_state: SystemState::Starting,
            system_state_time: 0.0,
            loading_progress: 0.0,
            background_frame_throttle: 4,
            background_frame_counter: 0,
        });

        let user_data = Box::into_raw(state) as *mut ffi::c_void;

        // convert config title to CString
        let title = CString::new(self.config.window_title).unwrap();

        sapp::run(&sapp::Desc {
            init_userdata_cb: Some(init::<T>),
            frame_userdata_cb: Some(frame::<T>),
            cleanup_userdata_cb: Some(cleanup::<T>),
            event_userdata_cb: Some(event::<T>),
            user_data,
            window_title: title.as_ptr(),
            width: self.config.window_width,
            height: self.config.window_height,
            sample_count: self.config.sample_count,
            high_dpi: self.config.high_dpi,
            logger: sapp::Logger { 
                func: Some(sokol::log::slog_func), 
                ..Default::default() 
            },
            icon: sapp::IconDesc { 
                sokol_default: true, 
                ..Default::default() 
            },
            ..Default::default()
        });
    }
}

extern "C" fn init<T: Game>(user_data: *mut ffi::c_void) {
    let state = unsafe { &mut *(user_data as *mut AppState<T>) };
    
    sg::setup(&sg::Desc {
        environment: sglue::environment(),
        logger: sg::Logger { 
            func: Some(sokol::log::slog_func), 
            ..Default::default() 
        },
        ..Default::default()
    });

    // Print backend info (helpful for debugging)
    let backend = sg::query_backend();
    match &backend {
        sg::Backend::Glcore | sg::Backend::Gles3 => {
            println!("Using GL Backend: {:?}", backend);
        },
        sg::Backend::D3d11 => {
            println!("Using D3D11 Backend");
        },
        sg::Backend::MetalIos | sg::Backend::MetalMacos | sg::Backend::MetalSimulator => {
            println!("Using Metal Backend: {:?}", backend);
        },
        sg::Backend::Wgpu => {
            println!("Using WGPU Backend");
        },
        sg::Backend::Dummy => {
            println!("Using Dummy Backend");
        },
    }

    // Set up default pass action for clearing screen
    state.pass_action.colors[0] = sg::ColorAttachmentAction {
        load_action: sg::LoadAction::Clear,
        clear_value: sg::Color { r: 8.0, g: 0.0, b: 0.0, a: 0.8 },
        ..Default::default()
    };
    
    //  Init render
    state.renderer.init();

    // Set initial camera viewport
    state.camera.set_viewport_size(
        sapp::width() as f32, 
        sapp::height() as f32
    );

    // Let the game do its initialization
    let config = T::config();
    state.game.init(&config, &mut state.renderer, &mut state.animation_manager, &mut state.particle_systems);
}

extern "C" fn frame<T: Game>(user_data: *mut ffi::c_void) {
    let state = unsafe { &mut *(user_data as *mut AppState<T>) };
    let dt = sapp::frame_duration() as f32;

    if state.system_state != SystemState::Starting {
        if let Some(requested_state) = state.game.request_system_state() {
            // Validate the transition
            if is_valid_transition(state.system_state, requested_state) {
                change_system_state(state, requested_state);
                return;
            } else {
                println!("Invalid state transition: {:?} -> {:?}", state.system_state, requested_state);
            }
        }
    }
    
    // Handle state-specific updates
    match state.system_state {
        SystemState::Starting => {
            state.system_state_time += dt;
            state.loading_progress = (state.system_state_time / 5.0).min(1.0);
            
            if state.loading_progress >= 1.0 {
                change_system_state(state, SystemState::GameActive);
            }
            
            render_loading_screen(state);
        }
        
        SystemState::GameActive => {
            // Full game update and render
            update_and_render_game(state, dt);
        }
        
        SystemState::Background => {
            state.background_frame_counter += 1;
            
            // Throttled updates - only every 4th frame
            if state.background_frame_counter % state.background_frame_throttle == 0 {
                // Reduced game update
                state.game.update(dt * state.background_frame_throttle as f32, &state.input, 
                                &mut state.camera, &mut state.animation_manager, &mut state.particle_systems);
                
                // Optional minimal rendering (or skip entirely)
                render_background_state(state);
            }
            // Note: Game can still request state changes via request_system_state()
        }
        
        SystemState::Shutdown => {
            sapp::request_quit();
        }
    }
        
    state.input.new_frame();
}

extern "C" fn cleanup<T: Game>(user_data: *mut ffi::c_void) {
    sg::shutdown();
    let _state = unsafe { Box::from_raw(user_data as *mut AppState<T>) };
    // State will be dropped automatically, cleaning up the game
}

extern "C" fn event<T: Game>(event: *const sapp::Event, user_data: *mut ffi::c_void) {
    let state = unsafe { &mut *(user_data as *mut AppState<T>) };
    let event = unsafe { &*event };
    
    // Engine handles system events
    match event._type {
        sapp::EventType::Suspended => {
            println!("System suspended");
            change_system_state(state, SystemState::Background);
            return;
        }
        sapp::EventType::Resumed => {
            println!("System resumed");
            // Only resume if we were actually in background
            if state.system_state == SystemState::Background {
                change_system_state(state, SystemState::GameActive);
            }
            return;
        }
        _ => {}
    }
    
    match state.system_state {
        SystemState::GameActive => {
            // Process input for InputManager
            process_input_events(state, event);
            
            // Pass event to game
            state.game.handle_event(event);
        }
        SystemState::Background => {
            // Limited event processing in background
            match event._type {
                sapp::EventType::Resized => {
                    state.camera.set_viewport_size(event.window_width as f32, event.window_height as f32);
                }
                _ => {
                    // Still pass to game, but game should handle appropriately
                    state.game.handle_event(event);
                }
            }
        }
        SystemState::Starting => {
            // No game events during startup
            match event._type {
                sapp::EventType::Resized => {
                    state.camera.set_viewport_size(event.window_width as f32, event.window_height as f32);
                }
                _ => {}
            }
        }
        SystemState::Shutdown => {
            // No event processing during shutdown
        }
    }
    
    // Process other events based on current state
    match state.system_state {
        SystemState::GameActive => {
            // Process input for InputManager
            process_input_events(state, event);
            
            // Pass event to game
            state.game.handle_event(event);
        }
        SystemState::Background => {
            // Limited event processing in background
            match event._type {
                sapp::EventType::Resized => {
                    state.camera.set_viewport_size(event.window_width as f32, event.window_height as f32);
                }
                _ => {
                    // Still pass to game, but game should handle appropriately
                    state.game.handle_event(event);
                }
            }
        }
        SystemState::Starting => {
            // No game events during startup
            match event._type {
                sapp::EventType::Resized => {
                    state.camera.set_viewport_size(event.window_width as f32, event.window_height as f32);
                }
                _ => {}
            }
        }
        SystemState::Shutdown => {
            // No event processing during shutdown
        }
    }
}

// -- system helpers --
fn update_and_render_game<T: Game>(state: &mut AppState<T>, dt: f32) {
    // Full game update
    for system in state.particle_systems.values_mut() {
        system.update(dt);
    }

    // Remove finished, duration-based systems
    let finished_keys: Vec<String> = state.particle_systems
        .iter()
        .filter_map(|(k, s)| if s.is_finished() { Some(k.clone()) } else { None })
        .collect();
    for key in finished_keys {
        state.particle_systems.remove(&key);
    }
    
    state.game.update(dt, &state.input, &mut state.camera, 
                    &mut state.animation_manager, &mut state.particle_systems);
    
    state.camera.update_shake(dt);

    // Update background color if needed
    if let Some(new_color) = state.game.request_background_color_change() {
        state.pass_action.colors[0].clear_value = new_color;
    }
    
    // Full rendering
    sg::begin_pass(&sg::Pass {
        action: state.pass_action,
        swapchain: sglue::swapchain(),
        ..Default::default()
    });
    
    state.game.render(&mut state.renderer, &mut state.camera);

    for system in state.particle_systems.values_mut() {
        for particle in system.get_particles() {
            state.renderer.draw_particle(particle);
        }
    }

    state.renderer.flush(&mut state.camera);

    sg::end_pass();
    sg::commit();
}

fn render_background_state<T: Game>(state: &mut AppState<T>) {
    // Minimal rendering or skip entirely for battery saving
    sg::begin_pass(&sg::Pass {
        action: state.pass_action,
        swapchain: sglue::swapchain(),
        ..Default::default()
    });
    
    // Could render at reduced quality or just clear screen
    state.game.render(&mut state.renderer, &mut state.camera);
    state.renderer.flush(&mut state.camera);
    
    sg::end_pass();
    sg::commit();
}

fn render_loading_screen<T: Game>(state: &mut AppState<T>) {
    sg::begin_pass(&sg::Pass {
        action: state.pass_action,
        swapchain: sglue::swapchain(),
        ..Default::default()
    });
    
    state.game.engine_render_loading(&mut state.renderer, state.loading_progress, &mut state.camera);
    state.renderer.flush(&mut state.camera);
    
    sg::end_pass();
    sg::commit();
}

fn is_valid_transition(from: SystemState, to: SystemState) -> bool {
    use SystemState::*;
    
    match (from, to) {
        (Starting, GameActive) => true,
        (Starting, _) => false,
        (GameActive, GameActive) => true,
        (GameActive, Background) => true,
        (GameActive, Shutdown) => true,
        (GameActive, Starting) => false,
        (Background, GameActive) => true,
        (Background, Shutdown) => true,
        (Background, _) => false,
        (Shutdown, _) => false,
        (a, b) if a == b => {
            true
        },
    }
}

fn process_input_events<T: Game>(state: &mut AppState<T>, event: &sapp::Event) {
    match event._type {
        sapp::EventType::KeyDown => state.input.handle_key_down(event.key_code),
        sapp::EventType::KeyUp => state.input.handle_key_up(event.key_code),
        sapp::EventType::MouseMove => state.input.handle_mouse_move(event.mouse_x, event.mouse_y),
        sapp::EventType::MouseDown => state.input.handle_mouse_button_down(event.mouse_button),
        sapp::EventType::MouseUp => state.input.handle_mouse_button_up(event.mouse_button),
        sapp::EventType::MouseScroll => state.input.handle_mouse_wheel(event.scroll_y),
        sapp::EventType::Resized => {
            state.camera.set_viewport_size(event.window_width as f32, event.window_height as f32);
        }
        _ => {}
    }
}

fn change_system_state<T: Game>(state: &mut AppState<T>, new_state: SystemState) {
    if state.system_state != new_state {
        println!("System state: {:?} -> {:?}", state.system_state, new_state);
        
        // Store previous state
        state.previous_system_state = state.system_state;
        state.system_state = new_state;
        state.system_state_time = 0.0;
        
        // State-specific initialization
        match new_state {
            SystemState::Background => {
                state.background_frame_counter = 0;
                println!("Entering background mode - throttling to every {} frames", 
                        state.background_frame_throttle);
            }
            SystemState::GameActive => {
                if state.previous_system_state == SystemState::Background {
                    println!("Resuming from background mode");
                }
            }
            SystemState::Shutdown => {
                println!("Initiating shutdown sequence");
            }
            _ => {}
        }
    }
}