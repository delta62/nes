use egui::{Context, InputState, Ui};
use nes::FrameBuffer;

pub trait View {
    /// Called whenever a window event occurs, such as pressing a key
    fn input(&mut self, input_state: &mut InputState) { }

    /// Called while rendering the main menu. Add menu items here.
    fn main_menu(&mut self, _ui: &mut Ui) { }

    /// Called each time a frame is rendered
    fn on_frame(&mut self, _framebuffer: &mut FrameBuffer) { }

    /// Called each time the NES CPU steps
    fn on_step(&mut self) { }

    /// Render an imgui window with controls
    fn window(&mut self, _ctx: &Context) { }

    /// Render a custom menu into the main menu
    fn custom_menu(&mut self, _ui: &mut Ui) { }

    /// Destroy the UI view
    fn destroy(&mut self) { }
}
