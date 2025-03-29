use egui::{Context, InputState, Ui};
use nes::{ControlMessage, ControlResponse, EmulationState, Frame};
use std::sync::mpsc::Sender;

pub trait View {
    /// Called whenever a window event occurs, such as pressing a key
    #[allow(unused_variables)]
    fn input(&mut self, input_state: &mut InputState) {}

    /// Called exactly once when a view is created for the first time,
    /// before any frames are run
    #[allow(unused_variables)]
    fn init(&mut self, control: &Sender<ControlMessage>) {}

    /// Called while rendering the main menu. Add menu items here.
    #[allow(unused_variables)]
    fn main_menu(&mut self, ui: &mut Ui) {}

    /// Called each time a frame is rendered
    #[allow(unused_variables)]
    fn on_frame(&mut self, frame: &Frame, ctrl: &Sender<ControlMessage>) {}

    /// Called each time the emulation state of the NES changes
    #[allow(unused_variables)]
    fn on_state_change(&mut self, state: EmulationState) {}

    /// Called when the console responds to a request for control data
    #[allow(unused_variables)]
    fn on_control_response(&mut self, message: &ControlResponse) {}

    /// Render an imgui window with controls
    #[allow(unused_variables)]
    fn window(&mut self, ctx: &Context) {}

    /// Render a custom menu into the main menu
    #[allow(unused_variables)]
    fn custom_menu(&mut self, ui: &mut Ui, ctx: &Context) {}
}
