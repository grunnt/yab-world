use crate::*;
use floating_duration::TimeAsFloat;
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::{dpi::PhysicalPosition, event::MouseScrollDelta};
use glutin::{
    event::{ElementState, Event, WindowEvent},
    platform::windows::WindowBuilderExtWindows,
};
use glutin::{Api, ContextBuilder, GlRequest};
use log::*;
use std::time::Instant;
use video::GlUtils;

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
        info!("Initializing system context");
        let events_loop = EventLoop::new();
        let wb = WindowBuilder::new()
            .with_title(title)
            .with_resizable(resizable)
            .with_min_inner_size(glutin::dpi::LogicalSize::new(1024, 768))
            .with_drag_and_drop(false) // Needs to be disabled to work with audio library
            .with_inner_size(glutin::dpi::LogicalSize::new(width, height));
        let windowed_context = ContextBuilder::new()
            .with_gl(GlRequest::Specific(
                Api::OpenGl,
                (opengl_major, opengl_minor),
            ))
            .with_vsync(true)
            .build_windowed(wb, &events_loop)
            .unwrap();
        let windowed_context = unsafe { windowed_context.make_current().unwrap() };

        info!(
            "Pixel format of the window's GL context: {:?}",
            windowed_context.get_pixel_format()
        );
        let gl_context = windowed_context.context();
        let gl = gl::Gl::load_with(|ptr| gl_context.get_proc_address(ptr) as *const _);
        info!("OpenGL version {}", gl.get_version());

        let scale_factor = 1.0;
        let mut context = SystemContext::new(&gl, width, height, scale_factor, assets);
        data.initialize(&mut context);

        setup(&mut data, &mut context);

        let mut state_manager = StateManager::new(data);
        state_manager.activate(starting_state, &mut context);

        let mut time = Instant::now();

        let mut input_events = Vec::new();
        let mut mouse_captured = false;
        context.input_mut().set_mouse_captured(mouse_captured);
        windowed_context
            .window()
            .set_cursor_grab(mouse_captured)
            .unwrap();
        windowed_context
            .window()
            .set_cursor_visible(!mouse_captured);
        events_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;
            match event {
                Event::LoopDestroyed => return,
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(physical_size) => {
                        if physical_size.width > 0 && physical_size.height > 0 {
                            windowed_context.resize(physical_size);
                            context
                                .video_mut()
                                .resize(physical_size.width, physical_size.height);
                            state_manager.resize(&mut context);
                        }
                    }
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::CursorMoved { position, .. } => {
                        let mut new_pos = Position::new(position.x as f32, position.y as f32);
                        if let Some(pos) = context.input().get_mouse_position() {
                            if mouse_captured {
                                // Get relative mouse movement while cursor stays in center
                                let size = Size::new(
                                    context.video().width() as f32,
                                    context.video().height() as f32,
                                );
                                let cx = size.width as f32 / 2.0;
                                let cy = size.height as f32 / 2.0;
                                let dx = cx - new_pos.x;
                                let dy = cy - new_pos.y;
                                if dx != 0.0 || dy != 0.0 {
                                    if let Ok(_) = windowed_context
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
                        context.input_mut().set_mouse_position(new_pos);
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
                                        context.input_mut().set_key_pressed(key, true);
                                        let shift = context.input().key_pressed(Key::LShift)
                                            || context.input().key_pressed(Key::RShift);
                                        input_events.push(InputEvent::KeyPress { key, shift });
                                    }
                                    ElementState::Released => {
                                        context.input_mut().set_key_pressed(key, false);
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
                                    context.input_mut().set_mouse_button_state(button, true);
                                }
                                ElementState::Released => {
                                    if let Some(pos) = context.input().get_mouse_position() {
                                        input_events.push(InputEvent::MouseClick {
                                            x: pos.x,
                                            y: pos.y,
                                            button,
                                        });
                                    }
                                    context.input_mut().set_mouse_button_state(button, false)
                                }
                            }
                        }
                    }
                    _ => (),
                },
                Event::MainEventsCleared => {
                    let delta_duration = time.elapsed();
                    let delta_s = delta_duration.as_fractional_secs() as f32;
                    time = Instant::now();

                    if mouse_captured != context.input().get_mouse_captured() {
                        mouse_captured = context.input().get_mouse_captured();
                        windowed_context
                            .window()
                            .set_cursor_grab(mouse_captured)
                            .unwrap();
                        windowed_context
                            .window()
                            .set_cursor_visible(!mouse_captured);
                        if mouse_captured {
                            // Prevent screen jumping after mouse capture is activated by setting cursor in center
                            windowed_context
                                .window()
                                .set_cursor_position(PhysicalPosition::new(
                                    context.video().width() as f32 / 2.0,
                                    context.video().height() as f32 / 2.0,
                                ))
                                .unwrap();
                        }
                        debug!("mouse_captured: {}", mouse_captured);
                    }

                    context.update_profile_mut().start();
                    if state_manager.update(delta_s, &input_events, &mut context) {
                        *control_flow = ControlFlow::Exit;
                    }
                    context.update_profile_mut().end();

                    input_events.clear();

                    if *control_flow != ControlFlow::Exit {
                        context.render_profile_mut().start();
                        context.video().clear_screen();
                        state_manager.render(&mut context);
                        context.render_profile_mut().end();
                        context.swap_profile_mut().start();
                        windowed_context.swap_buffers().unwrap();
                        context.swap_profile_mut().end();
                    }

                    // Update profiling for frame
                    context.frame_profile_mut().end();
                    context.render_profile_mut().frame();
                    context.swap_profile_mut().frame();
                    context.update_profile_mut().frame();
                    context.frame_profile_mut().frame();
                    context.frame_profile_mut().start();
                }
                _ => (),
            }
        });
    }
}
