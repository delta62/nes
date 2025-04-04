use super::View;
use egui::{
    load::SizedTexture, os::OperatingSystem, Color32, ColorImage, Context, KeyboardShortcut,
    ModifierNames, Sense, TextureHandle, TextureOptions, Vec2,
};
use nes::{ControlMessage, ControlRequest, ControlResponse, EmulationState};
use std::sync::mpsc::Sender;

const SHORTCUT: KeyboardShortcut = shortcut!(ALT, T);
const TABLE_WIDTH: usize = 128; // 16 tiles wide, 8px per tile
const TABLE_HEIGHT: usize = 128; // 16 tiles tall, 8px per tile

#[derive(Eq, PartialEq, Copy, Clone)]
enum PatternTableIndex {
    Table0000,
    Table1000,
}

pub struct PatternView {
    opened: bool,
    table1: Option<TextureHandle>,
    table2: Option<TextureHandle>,
    state: EmulationState,
    selected_table: PatternTableIndex,
}

impl PatternView {
    pub fn new(state: EmulationState) -> Self {
        let opened = false;
        let selected_table = PatternTableIndex::Table0000;
        let table1 = None;
        let table2 = None;

        Self {
            opened,
            state,
            table1,
            table2,
            selected_table,
        }
    }

    fn update_pattern_table(&mut self, table_index: PatternTableIndex, bytes: &[u8]) {
        let image = ColorImage::from_rgb([TABLE_WIDTH, TABLE_HEIGHT], bytes);
        match table_index {
            PatternTableIndex::Table0000 => self
                .table1
                .as_mut()
                .map(|tex| tex.set(image, TextureOptions::NEAREST)),
            PatternTableIndex::Table1000 => self
                .table2
                .as_mut()
                .map(|tex| tex.set(image, TextureOptions::NEAREST)),
        };
    }
}

impl View for PatternView {
    fn init(&mut self, ctx: &Context, control: &Sender<nes::ControlMessage>) {
        let t1_img = egui::ColorImage::new([TABLE_WIDTH, TABLE_HEIGHT], Color32::BLACK);
        self.table1 = Some(ctx.load_texture("pattern-table-lo", t1_img, Default::default()));

        let t2_img = egui::ColorImage::new([TABLE_WIDTH, TABLE_HEIGHT], Color32::BLACK);
        self.table2 = Some(ctx.load_texture("pattern-table-hi", t2_img, Default::default()));

        let req = ControlRequest::PatternTableContents;
        let msg = ControlMessage::ControlRequest(req);
        let _ = control.send(msg);
    }

    fn on_control_response(&mut self, message: &ControlResponse) {
        if let ControlResponse::PatternTableContents(contents) = message {
            self.update_pattern_table(PatternTableIndex::Table0000, &contents.table1);
            self.update_pattern_table(PatternTableIndex::Table1000, &contents.table2);
        }
    }

    fn window(&mut self, ctx: &egui::Context) {
        let opened = &mut self.opened;
        let selected_table = &mut self.selected_table;
        let state = self.state;
        let table1 = &self.table1;
        let table2 = &self.table2;

        egui::Window::new("Pattern Tables")
            .collapsible(false)
            .open(opened)
            .resizable(true)
            .default_size([256f32, 256f32])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .selectable_label(*selected_table == PatternTableIndex::Table0000, "0x0000")
                        .clicked()
                    {
                        *selected_table = PatternTableIndex::Table0000;
                    }

                    if ui
                        .selectable_label(*selected_table == PatternTableIndex::Table1000, "0x1000")
                        .clicked()
                    {
                        *selected_table = PatternTableIndex::Table1000;
                    }
                });

                if let EmulationState::Run(_) = state {
                    ui.label("Emulation is running");
                    return;
                }

                let tex = if *selected_table == PatternTableIndex::Table0000 {
                    &table1
                } else {
                    &table2
                };

                if let Some(tex) = tex {
                    let avail_width = ui.available_width();
                    let tex =
                        SizedTexture::new(tex.id(), [TABLE_WIDTH as f32, TABLE_HEIGHT as f32]);
                    let image = egui::Image::new(tex)
                        .maintain_aspect_ratio(true)
                        .fit_to_exact_size(Vec2::new(avail_width, avail_width))
                        .sense(Sense::drag());

                    ui.add(image);
                }
            });
    }

    fn main_menu(&mut self, ui: &mut egui::Ui) {
        let button = egui::Button::new("Pattern Table")
            .selected(self.opened)
            .shortcut_text(SHORTCUT.format(
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
