use super::View;
use chrono::Local;
use egui::{
    os::OperatingSystem, Button, DragValue, KeyboardShortcut, Layout, ModifierNames, ScrollArea,
};
use log::{error, info};
use nes::ControlMessage;
use std::{collections::VecDeque, sync::mpsc::Sender};

const SHORTCUT: KeyboardShortcut = shortcut!(ALT, L);
const DEFAULT_LOG_RETENTION: usize = 256;

pub struct LogView {
    capacity: usize,
    ctrl: Sender<ControlMessage>,
    logs: VecDeque<String>,
    opened: bool,
    scroll_to_bottom: bool,
    logging_enabled: bool,
}

impl LogView {
    pub fn new(ctrl: Sender<ControlMessage>) -> Self {
        let capacity = DEFAULT_LOG_RETENTION;
        let logs = VecDeque::with_capacity(capacity);
        let opened = false;
        let scroll_to_bottom = true;
        let logging_enabled = false;

        Self {
            capacity,
            ctrl,
            logs,
            logging_enabled,
            opened,
            scroll_to_bottom,
        }
    }

    fn export(&self) {
        let now = Local::now();
        let path = now.format("nes-logs-%Y-%m-%d-%H-%M-%S.log").to_string();
        let lines: Vec<_> = self.logs.iter().cloned().collect();
        let contents = lines.join("\n");
        info!("Exporting {} logs to {}", self.logs.len(), path);

        if let Err(err) = std::fs::write(path, contents) {
            error!("Error exporting logs: {}", err);
        }
    }
}

impl View for LogView {
    fn window(&mut self, ctx: &egui::Context) {
        let opened = &mut self.opened;
        let logs = &mut self.logs;
        let capacity = &mut self.capacity;
        let mut should_export = false;
        let scroll_to_bottom = &mut self.scroll_to_bottom;
        let logging_enabled = &mut self.logging_enabled;
        let ctrl = &self.ctrl;

        egui::Window::new("Logs")
            .collapsible(false)
            .min_size([690f32, 140f32])
            .open(opened)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Clear").clicked() {
                        logs.clear();
                    }

                    ui.label("Log count:");
                    if ui.add(DragValue::new(capacity).range(0..=10_000)).changed() {
                        while logs.len() > *capacity {
                            logs.pop_front();
                        }
                    }
                    if ui.button("Scroll to bottom").clicked() {
                        *scroll_to_bottom = true;
                    }

                    if ui
                        .checkbox(logging_enabled, "Enable logging (slow)")
                        .changed()
                    {
                        let msg = ControlMessage::EnableLogging(*logging_enabled);
                        let _ = ctrl.send(msg);
                    }

                    if ui.button("Export").clicked() {
                        should_export = true;
                    }
                });

                ui.separator();
                ui.add_space(4.0);

                ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                    ui.with_layout(
                        Layout::top_down(egui::Align::Min).with_cross_justify(true),
                        |ui| {
                            logs.iter().for_each(|log| {
                                ui.monospace(log);
                            });
                        },
                    );

                    if *scroll_to_bottom {
                        ui.scroll_to_cursor(Some(egui::Align::Max));
                        *scroll_to_bottom = false;
                    }
                });
            });

        if should_export {
            self.export();
        }
    }

    fn on_step(&mut self, log: &str, _ctrl: &Sender<ControlMessage>) {
        while self.logs.len() >= self.capacity {
            self.logs.pop_front();
        }

        self.logs.push_back(log.to_owned());
        self.scroll_to_bottom = true;
    }

    fn main_menu(&mut self, ui: &mut egui::Ui) {
        let button = Button::new("Logs").shortcut_text(SHORTCUT.format(
            &ModifierNames::NAMES,
            OperatingSystem::from_target_os() == OperatingSystem::Mac,
        ));

        if ui.add(button).clicked() {
            self.opened = !self.opened;
        }
    }

    fn input(&mut self, input_state: &mut egui::InputState) {
        if input_state.consume_shortcut(&SHORTCUT) {
            self.opened = !self.opened;
        }
    }
}
