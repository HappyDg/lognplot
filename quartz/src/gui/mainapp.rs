use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};

use std::sync::Arc;
use vulkano::device::Device;
use vulkano::framebuffer::RenderPassAbstract;

use super::visual1::MyVisual;

/// Application structure.
pub struct MainApp {
    device: Arc<Device>,
    visuals: Vec<MyVisual>,
    zoom_in: bool,
    zoom_out: bool,
    pub quit: bool,
}

impl MainApp {
    pub fn new(
        device: Arc<Device>,
        render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    ) -> Self {
        let my_visual = MyVisual::new(device.clone(), render_pass.clone(), 0.7_f32);
        let my_visual1 = MyVisual::new(device.clone(), render_pass.clone(), 0.3_f32);

        // array with visuals:
        let visuals = vec![my_visual, my_visual1];

        MainApp {
            device,
            visuals,
            zoom_in: false,
            zoom_out: false,
            quit: false,
        }
    }

    pub fn tick(&mut self) {
        if self.zoom_in {
            self.visuals[0].zoom *= 1.05_f32;
        }

        if self.zoom_out {
            self.visuals[0].zoom *= 0.95_f32;
        }
    }

    pub fn draw(
        &self,
        mut started_renderer: AutoCommandBufferBuilder,
        dynamic_state: &mut DynamicState,
    ) -> AutoCommandBufferBuilder {
        for visual in self.visuals.iter() {
            started_renderer = visual.draw(started_renderer, dynamic_state);
        }
        started_renderer
    }

    pub fn handle_event(&mut self, event: winit::WindowEvent) {
        match event {
            winit::WindowEvent::KeyboardInput { input, .. } => match input.state {
                winit::ElementState::Pressed => {
                    if let Some(virtual_keycode) = input.virtual_keycode {
                        self.handle_key_press(virtual_keycode)
                    }
                }
                winit::ElementState::Released => {
                    if let Some(virtual_keycode) = input.virtual_keycode {
                        self.handle_key_release(virtual_keycode);
                    }
                }
            },
            _ => {}
        }
    }

    fn handle_key_press(&mut self, virtual_keycode: winit::VirtualKeyCode) {
        match virtual_keycode {
            winit::VirtualKeyCode::D => {
                // info!("Zoom out pressed");
                self.zoom_out = true;
            }
            winit::VirtualKeyCode::S => {
                // info!("Zoom in");
                self.zoom_in = true;
            }
            winit::VirtualKeyCode::Q => {
                self.quit = true;
            }
            keycode => {
                info!("KEY {:?}", keycode);
            }
        }
    }

    fn handle_key_release(&mut self, virtual_keycode: winit::VirtualKeyCode) {
        match virtual_keycode {
            winit::VirtualKeyCode::D => {
                // info!("Zoom out");
                self.zoom_out = false;
            }
            winit::VirtualKeyCode::S => {
                // info!("Zoom in released");
                self.zoom_in = false;
            }
            _ => {}
        }
    }
}