use super::View;
use egui::{os::OperatingSystem, Context, KeyboardShortcut, ModifierNames, TextEdit, Ui};
use nes::{ControlMessage, ControlResponse, EmulationState, Flags, RegisterState};
use std::sync::mpsc::Sender;

const SHORTCUT: KeyboardShortcut = shortcut!(ALT, C);

pub struct CpuView {
    opened: bool,
    state: EmulationState,
    registers: RegisterState,
    ctrl: Sender<ControlMessage>,
    pc_addr: String,
    cpu_cycle: String,
}

impl CpuView {
    pub fn new(state: EmulationState, ctrl: Sender<ControlMessage>) -> Self {
        let opened = false;
        let registers = Default::default();
        let pc_addr = String::new();
        let cpu_cycle = String::new();

        Self {
            opened,
            ctrl,
            state,
            registers,
            pc_addr,
            cpu_cycle,
        }
    }
}

impl View for CpuView {
    fn init(&mut self, _ctx: &Context, control: &Sender<ControlMessage>) {
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
        let opened = &mut self.opened;
        let registers = &self.registers;
        let state = self.state;
        let pc_addr = &mut self.pc_addr;
        let cpu_cycle = &mut self.cpu_cycle;
        let ctrl = &self.ctrl;

        egui::Window::new("CPU")
            .collapsible(false)
            .open(opened)
            .show(ctx, |ui| {
                if let EmulationState::Run(_) = state {
                    ui.label("Emulation running");
                    return;
                }

                egui::Grid::new("cpuregs").striped(true).show(ui, |ui| {
                    ui.label("Cycle");
                    let cy = TextEdit::singleline(cpu_cycle);
                    if (ui.add(cy)).changed() {
                        if let Ok(cycle_count) = u64::from_str_radix(cpu_cycle, 10) {
                            let msg = ControlMessage::SetCpuCycles(cycle_count);
                            let _ = ctrl.send(msg);
                        }
                    }
                    ui.end_row();

                    ui.label("PC").on_hover_text("Program Counter");
                    let pc = TextEdit::singleline(pc_addr);
                    if ui.add(pc).changed() {
                        if let Ok(addr) = u16::from_str_radix(&pc_addr, 16) {
                            let msg = ControlMessage::SetProgramCounter(addr);
                            let _ = ctrl.send(msg);
                        }
                    }
                    ui.end_row();

                    ui.label("A").on_hover_text("Accumulator");
                    ui.label(format!("0x{:02x}", registers.a));
                    ui.end_row();
                    ui.label("X").on_hover_text("X Register");
                    ui.label(format!("0x{:02x}", registers.x));
                    ui.end_row();
                    ui.label("Y").on_hover_text("Y Register");
                    ui.label(format!("0x{:02x}", registers.y));
                    ui.end_row();
                    ui.label("S").on_hover_text("Stack Pointer");
                    ui.label(format!("0x{:02x}", registers.s));
                    ui.end_row();
                    ui.label("P").on_hover_text("Flags");
                    ui.label(format!("0x{:02x}", registers.p));
                    ui.end_row();
                });

                let flags = registers.p;

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

    fn on_step(&mut self, _log: &str, channel: &Sender<ControlMessage>) {
        if self.state == EmulationState::Pause || self.state == EmulationState::Step {
            let req = nes::ControlRequest::RegisterState;
            let _ = channel.send(ControlMessage::ControlRequest(req));
        }
    }

    fn on_control_response(&mut self, message: &ControlResponse) {
        if let ControlResponse::RegisterState(state) = message {
            self.registers = state.clone();
            self.pc_addr = format!("{:04X}", state.pc);
            self.cpu_cycle = format!("{}", state.cycle);
        }
    }

    fn on_state_change(&mut self, state: nes::EmulationState) {
        self.state = state;
    }
}
