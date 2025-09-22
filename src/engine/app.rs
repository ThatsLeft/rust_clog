use crate::engine::physics_world::PhysicsWorld;
use crate::engine::{
    AnimationManager, Camera2D, EngineServices, Game, GameConfig, InputManager, ParticleSystem,
    Renderer,
};
use sokol::{app as sapp, gfx as sg, glue as sglue};
use std::collections::HashMap;
use std::ffi::{self, CString};

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
    physics_world: PhysicsWorld,
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
            physics_world: PhysicsWorld::new(),
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
        }
        sg::Backend::D3d11 => {
            println!("Using D3D11 Backend");
        }
        sg::Backend::MetalIos | sg::Backend::MetalMacos | sg::Backend::MetalSimulator => {
            println!("Using Metal Backend: {:?}", backend);
        }
        sg::Backend::Wgpu => {
            println!("Using WGPU Backend");
        }
        sg::Backend::Dummy => {
            println!("Using Dummy Backend");
        }
    }

    // Set up default pass action for clearing screen
    state.pass_action.colors[0] = sg::ColorAttachmentAction {
        load_action: sg::LoadAction::Clear,
        clear_value: sg::Color {
            r: 8.0,
            g: 0.0,
            b: 0.0,
            a: 0.8,
        },
        ..Default::default()
    };

    //  Init render
    state.renderer.init();

    // Set initial camera viewport
    state
        .camera
        .set_viewport_size(sapp::width() as f32, sapp::height() as f32);

    let mut services = EngineServices {
        physics: &mut state.physics_world,
        particles: &mut state.particle_systems,
        animation: &mut state.animation_manager,
        camera: &mut state.camera,
        renderer: &mut state.renderer,
    };

    // Let the game do its initialization
    let config = T::config();
    state.game.init(&config, &mut services);
}

extern "C" fn frame<T: Game>(user_data: *mut ffi::c_void) {
    let state = unsafe { &mut *(user_data as *mut AppState<T>) };
    let dt = sapp::frame_duration() as f32;

    let mut services = EngineServices {
        physics: &mut state.physics_world,
        particles: &mut state.particle_systems,
        animation: &mut state.animation_manager,
        camera: &mut state.camera,
        renderer: &mut state.renderer,
    };

    // Game always updates and renders - no special loading path
    state.game.update(dt, &state.input, &mut services);
    services.update_camera_shake(dt);

    if let Some(new_color) = state.game.request_background_color_change() {
        state.pass_action.colors[0].clear_value = new_color;
    }

    // Single render path
    sg::begin_pass(&sg::Pass {
        action: state.pass_action,
        swapchain: sglue::swapchain(),
        ..Default::default()
    });

    state.game.render(&mut services);
    state.renderer.flush(&mut state.camera);

    sg::end_pass();
    sg::commit();

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

    process_input_events(state, event);
    state.game.handle_event(event);
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
            state
                .camera
                .set_viewport_size(event.window_width as f32, event.window_height as f32);
        }
        _ => {}
    }
}
