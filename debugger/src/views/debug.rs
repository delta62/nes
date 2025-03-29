use super::View;
use egui::{os::OperatingSystem, Context, KeyboardShortcut, ModifierNames, Ui};
use nes::{ControlMessage, EmulationState};
use std::sync::mpsc::Sender;

const SHOW_DEBUGGER: KeyboardShortcut = shortcut!(ALT, D);
const STEP_CPU: KeyboardShortcut = shortcut!(S);
const RUN: KeyboardShortcut = shortcut!(R);
const PAUSE: KeyboardShortcut = shortcut!(P);

pub struct DebugView {
    opened: bool,
    state: EmulationState,
    send_event: Sender<ControlMessage>,
}

impl DebugView {
    pub fn new(send_event: Sender<ControlMessage>) -> Self {
        let opened = false;
        let state = EmulationState::Pause;
        Self {
            opened,
            state,
            send_event,
        }
    }

    fn send(&self, state: EmulationState) {
        let msg = ControlMessage::SetState(state);
        let _ = self.send_event.send(msg);
    }
}

impl View for DebugView {
    fn main_menu(&mut self, ui: &mut Ui) {
        let button = egui::Button::new("Debugger")
            .selected(self.opened)
            .shortcut_text(SHOW_DEBUGGER.format(
                &ModifierNames::NAMES,
                OperatingSystem::from_target_os() == OperatingSystem::Mac,
            ));

        if ui.add(button).clicked() {
            self.opened = !self.opened;
        }
    }

    fn window(&mut self, ctx: &Context) {
        if !self.opened {
            return;
        }

        egui::Window::new("Debugger").auto_sized().show(ctx, |ui| {
            ui.label(format!("Emulation state: {:?}", self.state));

            egui::Grid::new("debugger").striped(true).show(ui, |ui| {
                ui.label(format!("{:?}", self.state));
                ui.end_row();
                ui.separator();
                ui.end_row();

                if ui.button("Step").clicked() {
                    self.send(EmulationState::Step);
                }

                if ui.button("Run").clicked() {
                    self.send(EmulationState::Run);
                }

                if ui.button("Pause").clicked() {
                    self.send(EmulationState::Pause);
                }
            });
        });
    }

    fn input(&mut self, input_state: &mut egui::InputState) {
        if input_state.consume_shortcut(&SHOW_DEBUGGER) {
            self.opened = !self.opened;
        }

        if input_state.consume_shortcut(&RUN) {
            self.send(EmulationState::Run);
        }

        if input_state.consume_shortcut(&PAUSE) {
            self.send(EmulationState::Pause);
        }

        if input_state.consume_shortcut(&STEP_CPU) {
            self.send(EmulationState::Step);
        }
    }

    fn custom_menu(&mut self, ui: &mut Ui, ctx: &Context) {
        ui.menu_button("Emulation", |ui| {
            let pause = egui::Button::new("Pause").shortcut_text(ctx.format_shortcut(&PAUSE));
            let play = egui::Button::new("Run").shortcut_text(ctx.format_shortcut(&PAUSE));
            let step_cpu = egui::Button::new("Step CPU").shortcut_text(ctx.format_shortcut(&PAUSE));

            ui.add(pause);
            ui.add(play);
            ui.add(step_cpu);
        });
    }
}
