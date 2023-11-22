use crate::views::{
    CpuView,
    // DebugView,
    // LogView,
    // NametableView,
    NesView,
    // OamView,
    // PaletteView,
    // PatternView,
    PpuView,
    // RecordView,
    // AudioRecordView,
    ScreenshotView,
    View
};
use nes::{Button, ButtonState, ControlMessage, EmulationState, FrameBuffer, VideoMessage};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender};

pub struct DebuggerWindow {
    frame_buffer: Arc<Mutex<FrameBuffer>>,
    receive_frame: Receiver<VideoMessage>,
    send_control: Sender<ControlMessage>,
    views: Vec<Box<dyn View>>,
}

impl eframe::App for DebuggerWindow {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.close();
                    }
                });

                ui.menu_button("Views", |ui| {
                    self.views.iter_mut().for_each(|v| v.main_menu(ui));
                });

                self.views.iter_mut().for_each(|v| v.custom_menu(ui));
            });
        });

        for view in self.views.iter_mut() {
            ctx.input_mut(|input| view.input(input));
            view.window(ctx);
        }
    }
}

impl DebuggerWindow {
    pub fn new(
        send_control: Sender<ControlMessage>,
        receive_frame: Receiver<VideoMessage>,
        frame_buffer: Arc<Mutex<FrameBuffer>>,
        record: bool,
        arecord: bool,
    ) -> Self {
        let views: Vec<Box<dyn View>> = vec![
            Box::new(CpuView::new()),
            // Box::new(DebugView::new()),
            Box::new(NesView::new()),
            Box::new(PpuView::new()),
            // Box::new(LogView::new()),
            // Box::new(PaletteView::new()),
            Box::new(ScreenshotView::new()),
            // Box::new(NametableView::new()),
            // Box::new(PatternView::new()),
            // Box::new(RecordView::new(record)),
            // Box::new(AudioRecordView::new(arecord)),
            // Box::new(OamView::new()),
        ];

        send_control.send(ControlMessage::SetState(EmulationState::Run)).unwrap();

        Self {
            frame_buffer,
            receive_frame,
            send_control,
            views,
        }
    }

    pub fn run(&mut self) {
        // while !self.window.should_close() {

        //     if let Ok(message) = self.receive_frame.try_recv() {
        //         match message {
        //             VideoMessage::FrameAvailable => {
        //                 let mut fb = self.frame_buffer.lock().expect("Can't lock framebuffer for reading");
        //                 self.views.iter_mut().for_each(|v| v.on_frame(&mut fb));
        //             }
        //         }
        //     }



        // self.glfw.poll_events();
        // for (_, event) in glfw::flush_messages(&self.events) {
        //     self.imgui_glfw.handle_event(&mut self.imgui, &event);

        //     views.iter_mut().for_each(|v| v.key_event(&event));

        //     match event {
        //         WindowEvent::Key(Key::Q, _, Action::Press, Modifiers::Control) => {
        //             self.window.set_should_close(true);
        //         }

        //         event => {
        //             let nes_button_press = to_nes(&event);

        //             if let Some((button, state)) = nes_button_press {
        //                 let message = ControlMessage::ControllerInput(button, state);
        //                 self.send_control.send(message).expect("NES thread hung up!");
        //             }
        //         }
        //     }
        // }

        self.views.iter_mut().for_each(|v| v.destroy());

        self.send_control.send(ControlMessage::SetState(EmulationState::Kill)).unwrap();
    }
}

// Convert a GLFW key event to an NES controller input event
// fn to_nes(event: &WindowEvent) -> Option<(Button, ButtonState)> {
//     if let WindowEvent::Key(key, _, action, _) = event {
//         let state = match action {
//             Action::Press   => Some(ButtonState::Press),
//             Action::Release => Some(ButtonState::Release),
//             Action::Repeat  => None,
//         };

//         let button = match key {
//             Key::Enter     => Some(Button::Start),
//             Key::Backspace => Some(Button::Select),
//             Key::Z         => Some(Button::A),
//             Key::X         => Some(Button::B),
//             Key::Up        => Some(Button::Up),
//             Key::Down      => Some(Button::Down),
//             Key::Left      => Some(Button::Left),
//             Key::Right     => Some(Button::Right),
//             _              => None,
//         };

//         match (button, state) {
//             (Some(b), Some(s)) => Some((b, s)),
//             _                  => None,
//         }
//     } else {
//         None
//     }
// }
