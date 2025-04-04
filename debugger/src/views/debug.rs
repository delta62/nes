use super::View;
use disasm::{AddressOrOp, Disassembler, Mnemonic, Op};
use egui::{
    os::OperatingSystem, Color32, Context, DragValue, Frame, KeyboardShortcut, ModifierNames,
    RichText, ScrollArea, TextEdit, TextStyle, Ui, Vec2,
};
use nes::{ControlMessage, ControlRequest, ControlResponse, EmulationState};
use std::sync::mpsc::Sender;

const SHOW_DEBUGGER: KeyboardShortcut = shortcut!(ALT, D);
const STEP_CPU: KeyboardShortcut = shortcut!(S);
const RUN_PAUSE: KeyboardShortcut = shortcut!(Space);

pub struct DebugView {
    opened: bool,
    state: EmulationState,
    send_event: Sender<ControlMessage>,
    breakpoints_enabled: bool,
    disasm: Option<Disassembler>,
    current_pc: u16,
    jump_to_pc: String,
    scroll_to: Option<usize>,
    instr_limit: usize,
}

impl DebugView {
    pub fn new(send_event: Sender<ControlMessage>) -> Self {
        let opened = false;
        let state = EmulationState::Pause;
        let breakpoints_enabled = true;
        let disasm = None;
        let current_pc = 0xFFFC;
        let jump_to_pc = String::new();
        let scroll_to = None;
        let instr_limit = 0;

        Self {
            opened,
            instr_limit,
            disasm,
            state,
            scroll_to,
            jump_to_pc,
            current_pc,
            send_event,
            breakpoints_enabled,
        }
    }

    fn send(&self, state: EmulationState) {
        let msg = ControlMessage::SetState(state);
        let _ = self.send_event.send(msg);
    }
}

impl View for DebugView {
    fn init(&mut self, _ctx: &Context, control: &Sender<ControlMessage>) {
        let req = ControlRequest::RomContents;
        let _ = control.send(ControlMessage::ControlRequest(req));
    }

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
        let opened = &mut self.opened;
        let state = self.state;
        let send_event = &self.send_event;
        let disasm = &self.disasm;
        let breakpoints_enabled = &mut self.breakpoints_enabled;
        let current_pc = self.current_pc;
        let jump_addr = &mut self.jump_to_pc;
        let scroll_to = &mut self.scroll_to;
        let instr_limit = &mut self.instr_limit;

        egui::Window::new("Debugger")
            .open(opened)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.label(format!("Emulation state: {:?}", state));
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Step").clicked() {
                        let msg = ControlMessage::SetState(EmulationState::Step);
                        let _ = send_event.send(msg);
                    }

                    if ui.button(if let EmulationState::Run(_) = state { "Pause" } else { "Run" }).clicked() {
                        if state == EmulationState::Kill {
                            return;
                        }

                        if let EmulationState::Run(_) = state {
                            let _ = send_event.send(ControlMessage::SetState(EmulationState::Pause));
                        } else {
                            let limit = if *instr_limit > 0 { Some(*instr_limit) } else { None };
                            let msg = ControlMessage::SetState(EmulationState::Run(limit));
                            let _ = send_event.send(msg);
                        }
                    }

                    ui.label("instructions");
                    ui.add(DragValue::new(instr_limit).range(0..=usize::MAX));

                    let jump_label = ui.label("Jump to");
                    ui.label("0x");
                    let jump_edit = TextEdit::singleline(jump_addr).desired_width(50.0);
                    if ui.add(jump_edit).labelled_by(jump_label.id).lost_focus() {
                        match u16::from_str_radix(jump_addr, 16) {
                            Ok(addr) => {
                                disasm.as_ref().map(|disasm| {
                                    *scroll_to = Some(disasm.index_of(addr));
                                });
                            }
                            Err(_) => {}
                        }
                    }

                    if ui.button("Jump to PC").clicked() {
                        disasm.as_ref().map(|disasm| {
                            *scroll_to = Some(disasm.index_of(current_pc));
                        });
                    }

                    ui.checkbox(breakpoints_enabled, "Enable breakpoints");
                });

                if let Some(disasm) = disasm {
                    let num_rows = disasm.len();
                    let row_height = ui.text_style_height(&TextStyle::Body);

                    let mut scroll = ScrollArea::vertical().auto_shrink(false);

                    if let Some(offset) = scroll_to {
                        scroll = scroll.vertical_scroll_offset(*offset as f32 * row_height);
                        *scroll_to = None;
                    }

                    ui.spacing_mut().item_spacing = Vec2::new(0.0, 0.0);

                    scroll.show_rows(ui, row_height, num_rows, |ui, row_range| {
                        for (i, addr) in row_range.enumerate() {
                            if let Some((addr, op)) = disasm.get(addr) {
                                ui.horizontal(|ui| {
                                    let avail_width = ui.available_width();
                                    let color = if current_pc == addr {
                                        Color32::DARK_BLUE
                                    } else if i % 2 == 0 {
                                        Color32::from_rgb(40, 40, 40)
                                    } else {
                                        Color32::TRANSPARENT
                                    };

                                    Frame::NONE.fill(color).show(ui, |ui| {
                                        ui.set_width(avail_width);
                                        ui.set_height(row_height);

                                        // Gutter for breakpoint
                                        ui.add_space(24.0);

                                        // Address
                                        let addr_label = RichText::new(format!("{:04X}", addr))
                                            .monospace()
                                            .color(Color32::DARK_GRAY);
                                        ui.label(addr_label);

                                        ui.add_space(8.0);

                                        // Operation
                                        match op {
                                            AddressOrOp::Unknown(opcode) => {
                                                let mnemonic = Mnemonic::from(opcode);
                                                let s = format!("{:02X} {}      ", opcode, mnemonic);
                                                let xyz = RichText::new(s)
                                                    .monospace()
                                                    .color(Color32::DARK_GRAY);
                                                ui.label(xyz);
                                            }
                                            AddressOrOp::Op(op) => {
                                                let Op { mnemonic, opcode, addr } = op;
                                                let byte1 = addr.byte1().map(|b| format!("{b:02X}")).unwrap_or_else(|| "  ".to_owned());
                                                let byte2 = addr.byte2().map(|b| format!("{b:02X}")).unwrap_or_else(|| "  ".to_owned());
                                                let s = format!(
                                                    "{opcode:02X} {byte1} {byte2} {mnemonic} {addr}",
                                                );
                                                ui.monospace(s);
                                            }
                                            AddressOrOp::Address(addr) => {
                                                ui.monospace(format!("{}", addr));
                                            }
                                        }
                                    });
                                });
                            }
                        }
                    });
                }
            });
    }

    fn on_control_response(&mut self, message: &ControlResponse) {
        match message {
            ControlResponse::RegisterState(state) => {
                self.current_pc = state.pc;
            }
            ControlResponse::RomContents(buf) => {
                self.disasm = Some(Disassembler::new(buf));
            }
            _ => {}
        }
    }

    fn input(&mut self, input_state: &mut egui::InputState) {
        if input_state.consume_shortcut(&SHOW_DEBUGGER) {
            self.opened = !self.opened;
        }

        if input_state.consume_shortcut(&RUN_PAUSE) {
            if self.state == EmulationState::Kill {
                return;
            }

            if let EmulationState::Run(_) = self.state {
                self.send(EmulationState::Pause);
            } else {
                self.send(EmulationState::Run(None));
            }
        }

        if input_state.consume_shortcut(&STEP_CPU) {
            self.send(EmulationState::Step);
        }
    }

    fn custom_menu(&mut self, ui: &mut Ui, ctx: &Context) {
        ui.menu_button("Emulation", |ui| {
            let run_pause = egui::Button::new(if let EmulationState::Run(_) = self.state {
                "Pause"
            } else {
                "Run"
            })
            .shortcut_text(ctx.format_shortcut(&RUN_PAUSE));
            let step_cpu =
                egui::Button::new("Step CPU").shortcut_text(ctx.format_shortcut(&STEP_CPU));

            ui.add(run_pause);
            ui.add(step_cpu);
        });
    }

    fn on_state_change(&mut self, state: EmulationState) {
        self.state = state;
    }
}
