use super::View;
use egui::{os::OperatingSystem, Context, KeyboardShortcut, ModifierNames, Ui};
use nes::{ControlMessage, ControlResponse, EmulationState, Flags, Frame, RegisterState};
use std::sync::mpsc::Sender;

const SHORTCUT: KeyboardShortcut = KeyboardShortcut {
    modifiers: egui::Modifiers::ALT,
    logical_key: egui::Key::C,
};

pub struct CpuView {
    opened: bool,
    state: EmulationState,
    registers: RegisterState,
}

impl CpuView {
    pub fn new(state: EmulationState) -> Self {
        let opened = false;
        let registers = Default::default();
        Self {
            opened,
            state,
            registers,
        }
    }
}

impl View for CpuView {
    fn init(&mut self, control: &Sender<ControlMessage>) {
        let reg = nes::ControlRequest::RegisterState;
        let msg = ControlMessage::ControlRequest(reg);
        let _ = control.send(msg);
    }

    fn main_menu(&mut self, ui: &mut Ui) {
        let button = egui::Button::new("CPU")
            .selected(self.opened)
            .shortcut_text(SHORTCUT.format(
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

        egui::Window::new("CPU").auto_sized().show(ctx, |ui| {
            if self.state == EmulationState::Run {
                ui.label("Emulation running");
                return;
            }

            egui::Grid::new("cpuregs").striped(true).show(ui, |ui| {
                ui.label("Cycle");
                ui.label(format!("{}", self.registers.cycle));
                ui.end_row();
                ui.label("PC").on_hover_text("Program Counter");
                ui.label(format!("0x{:04x}", self.registers.pc));
                ui.end_row();
                ui.label("A").on_hover_text("Accumulator");
                ui.label(format!("0x{:02x}", self.registers.a));
                ui.end_row();
                ui.label("X").on_hover_text("X Register");
                ui.label(format!("0x{:02x}", self.registers.x));
                ui.end_row();
                ui.label("Y").on_hover_text("Y Register");
                ui.label(format!("0x{:02x}", self.registers.y));
                ui.end_row();
                ui.label("S").on_hover_text("Stack Pointer");
                ui.label(format!("0x{:02x}", self.registers.s));
                ui.end_row();
                ui.label("P").on_hover_text("Flags");
                ui.label(format!("0x{:02x}", self.registers.p));
                ui.end_row();
            });

            let flags = self.registers.p;

            egui::Grid::new("cpuflags").show(ui, |ui| {
                ui.disable();
                ui.checkbox(&mut flags.contains(Flags::CARRY), "C")
                    .on_hover_text("Carry flag");
                ui.checkbox(&mut flags.contains(Flags::ZERO), "Z")
                    .on_hover_text("Zero flag");
                ui.checkbox(&mut flags.contains(Flags::IRQ_DISABLE), "I")
                    .on_hover_text("Interrupt disable flag");
                ui.checkbox(&mut flags.contains(Flags::DECIMAL), "D")
                    .on_hover_text("Decimal flag");
                ui.end_row();

                ui.checkbox(&mut flags.contains(Flags::BREAK), "B")
                    .on_hover_text("Break flag");
                ui.label("-");
                ui.checkbox(&mut flags.contains(Flags::OVERFLOW), "V")
                    .on_hover_text("Overflow flag");
                ui.checkbox(&mut flags.contains(Flags::NEGATIVE), "N")
                    .on_hover_text("Negative flag");
                ui.end_row();
            });
        });
    }

    fn input(&mut self, input_state: &mut egui::InputState) {
        if input_state.consume_shortcut(&SHORTCUT) {
            self.opened = !self.opened;
        }
    }

    fn on_frame(&mut self, _frame: &Frame, channel: &Sender<ControlMessage>) {
        if self.state == EmulationState::Pause {
            let req = nes::ControlRequest::RegisterState;
            let _ = channel.send(ControlMessage::ControlRequest(req));
        }
    }

    fn on_control_response(&mut self, message: &ControlResponse) {
        if let ControlResponse::RegisterState(state) = message {
            self.registers = state.clone();
        }
    }

    fn on_state_change(&mut self, state: nes::EmulationState) {
        self.state = state;
    }
}
