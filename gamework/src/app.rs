use crate::*;
use floating_duration::TimeAsFloat;
use glow::*;
use glutin::event::{ElementState, Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::platform::windows::WindowBuilderExtWindows;
use glutin::window::{CursorGrabMode, WindowBuilder};
use glutin::{dpi::PhysicalPosition, event::MouseScrollDelta};
use glutin::{Api, ContextBuilder, GlRequest};
use log::*;
use std::sync::Arc;
use std::time::Instant;

pub struct App {}

impl App {
    /// Create a new window and graphics context
    pub fn run<GameContext: 'static + SharedContext>(
        title: &str,
        width: u32,
        height: u32,
        resizable: bool,
        opengl_major: u8,
        opengl_minor: u8,
        starting_state: Box<dyn State<GameContext>>,
        assets: &Assets,
        mut data: GameContext,
        setup: Box<dyn Fn(&mut GameContext, &mut SystemContext) -> ()>,
    ) {
        let debug_opengl = true;

        info!("Initializing system context");
        let events_loop = EventLoop::new();
        let window_builder = WindowBuilder::new()
            .with_title(title)
            .with_resizable(resizable)
            .with_drag_and_drop(false) // Needs to be disabled to work with audio library
            .with_inner_size(glutin::dpi::LogicalSize::new(width, height));
        let window_context = unsafe {
            ContextBuilder::new()
                .with_gl(GlRequest::Specific(
                    Api::OpenGl,
                    (opengl_major, opengl_minor),
                ))
                .with_srgb(true)
                .with_vsync(true)
                .with_gl_debug_flag(debug_opengl)
                .build_windowed(window_builder, &events_loop)
                .unwrap()
                .make_current()
                .unwrap()
        };
        info!(
            "Pixel format of the window's GL context: {:?}",
            window_context.get_pixel_format()
        );
        let gl = unsafe {
            glow::Context::from_loader_function(|s| window_context.get_proc_address(s) as *const _)
        };
        let gl = Arc::new(gl);
        info!("OpenGL version {:?}", gl.version());

        if debug_opengl {
            enable_opengl_logging(&gl, true);
        }

        let mut egui_glow = egui_glow::EguiGlow::new(&events_loop, gl.clone());
        egui_glow.egui_ctx.set_pixels_per_point(2.0);

        let scale_factor = 1.0;
        let mut system = SystemContext::new(gl, width, height, scale_factor, assets);

        data.initialize(&mut system);

        setup(&mut data, &mut system);

        let mut state_manager = StateManager::new(data);
        state_manager.activate(starting_state, &mut system);

        let mut time = Instant::now();

        let mut input_events = Vec::new();
        let mut mouse_captured = false;
        system.input_mut().set_mouse_captured(mouse_captured);
        window_context
            .window()
            .set_cursor_grab(if mouse_captured {
                CursorGrabMode::Confined
            } else {
                CursorGrabMode::None
            })
            .unwrap();
        window_context.window().set_cursor_visible(!mouse_captured);
        debug!("Starting event loop");
        events_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::LoopDestroyed => {
                    egui_glow.destroy();
                    return;
                }
                Event::WindowEvent { event, .. } => {
                    match event {
                        WindowEvent::Resized(physical_size) => {
                            if physical_size.width > 0 && physical_size.height > 0 {
                                window_context.resize(physical_size);
                                system
                                    .video_mut()
                                    .resize(physical_size.width, physical_size.height);
                                state_manager.resize(&mut system);
                            }
                        }
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::CursorMoved { position, .. } => {
                            let mut new_pos = Position::new(position.x as f32, position.y as f32);
                            if let Some(pos) = system.input().get_mouse_position() {
                                if mouse_captured {
                                    // Get relative mouse movement while cursor stays in center
                                    let size = Size::new(
                                        system.video().width() as f32,
                                        system.video().height() as f32,
                                    );
                                    let cx = size.width as f32 / 2.0;
                                    let cy = size.height as f32 / 2.0;
                                    let dx = cx - new_pos.x;
                                    let dy = cy - new_pos.y;
                                    if dx != 0.0 || dy != 0.0 {
                                        if let Ok(_) = window_context
                                            .window()
                                            .set_cursor_position(PhysicalPosition::new(cx, cy))
                                        {
                                            new_pos.x = cx;
                                            new_pos.y = cy;
                                            input_events.push(InputEvent::MouseMove {
                                                x: cx,
                                                y: cy,
                                                dx,
                                                dy,
                                            });
                                        }
                                    }
                                } else {
                                    let dx = pos.x - new_pos.x;
                                    let dy = pos.y - new_pos.y;
                                    if dx != 0.0 || dy != 0.0 {
                                        input_events.push(InputEvent::MouseMove {
                                            x: new_pos.x,
                                            y: new_pos.y,
                                            dx,
                                            dy,
                                        });
                                    }
                                }
                            }
                            system.input_mut().set_mouse_position(new_pos);
                        }
                        WindowEvent::MouseWheel { delta, .. } => {
                            match delta {
                                MouseScrollDelta::PixelDelta(PhysicalPosition { y, .. }) => {
                                    input_events.push(InputEvent::MouseScroll { delta: y as f32 });
                                }
                                MouseScrollDelta::LineDelta(y, ..) => {
                                    input_events.push(InputEvent::MouseScroll { delta: y as f32 });
                                }
                            };
                        }
                        WindowEvent::KeyboardInput { input, .. } => {
                            if let Some(key_code) = input.virtual_keycode {
                                if let Some(key) = code_to_key(&key_code) {
                                    match input.state {
                                        ElementState::Pressed => {
                                            system.input_mut().set_key_pressed(key, true);
                                            let shift = system.input().key_pressed(Key::LShift)
                                                || system.input().key_pressed(Key::RShift);
                                            input_events.push(InputEvent::KeyPress { key, shift });
                                        }
                                        ElementState::Released => {
                                            system.input_mut().set_key_pressed(key, false);
                                        }
                                    }
                                }
                            }
                        }
                        WindowEvent::MouseInput { button, state, .. } => {
                            let button_opt = match button {
                                glutin::event::MouseButton::Left => Some(MouseButton::Left),
                                glutin::event::MouseButton::Right => Some(MouseButton::Right),
                                glutin::event::MouseButton::Middle => Some(MouseButton::Middle),
                                _ => None,
                            };
                            if let Some(button) = button_opt {
                                match state {
                                    ElementState::Pressed => {
                                        system.input_mut().set_mouse_button_state(button, true);
                                    }
                                    ElementState::Released => {
                                        if let Some(pos) = system.input().get_mouse_position() {
                                            input_events.push(InputEvent::MouseClick {
                                                x: pos.x,
                                                y: pos.y,
                                                button,
                                            });
                                        }
                                        system.input_mut().set_mouse_button_state(button, false)
                                    }
                                }
                            }
                        }
                        _ => (),
                    }
                    egui_glow.on_event(&event);
                }
                Event::MainEventsCleared => {
                    let delta_duration = time.elapsed();
                    let delta_s = delta_duration.as_fractional_secs() as f32;
                    time = Instant::now();

                    if mouse_captured != system.input().get_mouse_captured() {
                        mouse_captured = system.input().get_mouse_captured();
                        window_context
                            .window()
                            .set_cursor_grab(if mouse_captured {
                                CursorGrabMode::Confined
                            } else {
                                CursorGrabMode::None
                            })
                            .unwrap();
                        window_context.window().set_cursor_visible(!mouse_captured);
                        if mouse_captured {
                            // Prevent screen jumping after mouse capture is activated by setting cursor in center
                            window_context
                                .window()
                                .set_cursor_position(PhysicalPosition::new(
                                    system.video().width() as f32 / 2.0,
                                    system.video().height() as f32 / 2.0,
                                ))
                                .unwrap();
                        }
                        debug!("mouse_captured: {}", mouse_captured);
                    }

                    system.update_profile_mut().start();
                    egui_glow.run(window_context.window(), |egui_ctx| {
                        if state_manager.update(delta_s, egui_ctx, &input_events, &mut system) {
                            *control_flow = ControlFlow::Exit;
                        }
                    });
                    system.update_profile_mut().end();

                    input_events.clear();

                    if *control_flow != ControlFlow::Exit {
                        system.render_profile_mut().start();
                        system.video().clear_screen();
                        state_manager.render(&mut system);
                        system.render_profile_mut().end();

                        egui_glow.paint(window_context.window());

                        system.swap_profile_mut().start();
                        window_context.swap_buffers().unwrap();
                        system.swap_profile_mut().end();
                    }

                    // Update profiling for frame
                    system.frame_profile_mut().end();
                    system.render_profile_mut().frame();
                    system.swap_profile_mut().frame();
                    system.update_profile_mut().frame();
                    system.frame_profile_mut().frame();
                    system.frame_profile_mut().start();
                }
                _ => (),
            }
        });
    }
}

// Install a debug logging callback in the OpenGL context. Synchronous mode helps finding error causes but is much slower.
fn enable_opengl_logging(gl: &Context, synchronous: bool) {
    let callback = |gl_source: u32, gl_type: u32, gl_id: u32, severity: u32, text: &str| {
        let log_level = match severity {
            glow::DEBUG_SEVERITY_HIGH => Level::Error,
            glow::DEBUG_SEVERITY_MEDIUM => Level::Warn,
            glow::DEBUG_SEVERITY_LOW => Level::Info,
            _ => Level::Debug,
        };
        if log_level == Level::Debug {
            return;
        }
        let gl_source = match gl_source {
            glow::DEBUG_SOURCE_API => "API",
            glow::DEBUG_SOURCE_APPLICATION => "application",
            glow::DEBUG_SOURCE_SHADER_COMPILER => "shader compiler",
            glow::DEBUG_SOURCE_THIRD_PARTY => "third party",
            glow::DEBUG_SOURCE_WINDOW_SYSTEM => "window system",
            _ => "other",
        };
        let gl_type = match gl_type {
            glow::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "deprecated behaviour",
            glow::DEBUG_TYPE_ERROR => "error",
            glow::DEBUG_TYPE_MARKER => "marker",
            glow::DEBUG_TYPE_PERFORMANCE => "performance",
            glow::DEBUG_TYPE_PORTABILITY => "portability",
            glow::DEBUG_TYPE_PUSH_GROUP => "push group",
            glow::DEBUG_TYPE_POP_GROUP => "pop group",
            glow::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "undefined behaviour",
            _ => "other",
        };
        log!(
            log_level,
            "OpenGL {} {}: {} ({:x}).",
            gl_source,
            gl_type,
            text,
            gl_id
        );
        if Level::Error == log_level {
            panic!("OpenGL {} {}: {} ({:x}).", gl_source, gl_type, text, gl_id);
        }
    };
    unsafe {
        gl.enable(glow::DEBUG_OUTPUT);
        if synchronous {
            gl.enable(glow::DEBUG_OUTPUT_SYNCHRONOUS);
        }
        gl.debug_message_callback(callback);
    }
}
