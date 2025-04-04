use crate::views::{
    CpuView,
    DebugView,
    LogView,
    // NametableView,
    NesView,
    // OamView,
    PaletteView,
    PatternView,
    PpuView,
    // RecordView,
    // AudioRecordView,
    RomView,
    ScreenshotView,
    View,
};
use egui::{Button, KeyboardShortcut};
use nes::{ControlMessage, EmulationState, INesHeader, VideoMessage};
use std::{
    path::PathBuf,
    sync::mpsc::{Receiver, Sender},
};

const QUIT_SHORTCUT: KeyboardShortcut = shortcut!(CTRL, Q);

pub struct DebuggerWindow {
    receive_frame: Receiver<VideoMessage>,
    send_control: Sender<ControlMessage>,
    views: Vec<Box<dyn View>>,
    first_update: bool,
}

fn exit(ctx: &egui::Context) {
    let ctx = ctx.clone();
    std::thread::spawn(move || ctx.send_viewport_cmd(egui::ViewportCommand::Close));
}

impl eframe::App for DebuggerWindow {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        if self.first_update {
            let views = &mut self.views;
            let ctrl = &self.send_control;

            views.iter_mut().for_each(|view| view.init(ctx, ctrl));
            self.first_update = false;
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    let button =
                        Button::new("Quit").shortcut_text(ctx.format_shortcut(&QUIT_SHORTCUT));

                    if ui.add(button).clicked() {
                        exit(ctx)
                    }
                });

                ui.menu_button("Views", |ui| {
                    self.views.iter_mut().for_each(|v| v.main_menu(ui));
                });

                self.views.iter_mut().for_each(|v| v.custom_menu(ui, ctx));
            });
        });

        ctx.input_mut(|i| {
            if i.consume_shortcut(&QUIT_SHORTCUT) {
                exit(ctx);
            }
        });

        loop {
            match self.receive_frame.try_recv() {
                Ok(VideoMessage::FrameAvailable(frame)) => {
                    let views = &mut self.views;
                    let ctrl = &self.send_control;
                    views.iter_mut().for_each(|v| v.on_frame(&frame, ctrl));
                    let _ = self.send_control.send(ControlMessage::RecycleFrame(frame));
                }
                Ok(VideoMessage::ControlResponse(res)) => {
                    self.views
                        .iter_mut()
                        .for_each(|v| v.on_control_response(&res));
                }
                Ok(VideoMessage::StateChanged(new_state)) => {
                    self.views
                        .iter_mut()
                        .for_each(|v| v.on_state_change(new_state));
                }
                Ok(VideoMessage::CpuStep(log_line)) => {
                    let views = &mut self.views;
                    let send_control = &self.send_control;
                    views
                        .iter_mut()
                        .for_each(|v| v.on_step(log_line.as_ref(), send_control));
                }
                Err(_) => break,
            }
        }

        for view in self.views.iter_mut() {
            ctx.input_mut(|input| view.input(input));
            view.window(ctx);
        }

        ctx.request_repaint();
    }
}

impl DebuggerWindow {
    pub fn new<P: Into<PathBuf>>(
        initial_state: EmulationState,
        send_control: Sender<ControlMessage>,
        receive_frame: Receiver<VideoMessage>,
        rom_header: INesHeader,
        rom_path: P,
        _record: bool,
        _arecord: bool,
    ) -> Self {
        let first_update = true;
        let views: Vec<Box<dyn View>> = vec![
            Box::new(CpuView::new(initial_state, send_control.clone())),
            Box::new(NesView::new(send_control.clone())),
            Box::new(PpuView::new(initial_state)),
            Box::new(LogView::new(send_control.clone())),
            Box::new(RomView::new(rom_path, rom_header)),
            Box::new(ScreenshotView::new()),
            Box::new(PatternView::new(initial_state)),
            Box::new(PaletteView::new(initial_state)),
            // Box::new(NametableView::new()),
            // Box::new(RecordView::new(record)),
            // Box::new(AudioRecordView::new(arecord)),
            // Box::new(OamView::new()),

            // Last, because step shortcuts will eat ALT- prefixed keycodes otherwise
            Box::new(DebugView::new(send_control.clone())),
        ];

        Self {
            receive_frame,
            send_control,
            views,
            first_update,
        }
    }
}
