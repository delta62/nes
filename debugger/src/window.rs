use crate::views::{
    // CpuView,
    // DebugView,
    // LogView,
    // NametableView,
    NesView,
    // OamView,
    // PaletteView,
    // PatternView,
    // PpuView,
    RecordView,
    AudioRecordView,
    ScreenshotView,
    View
};
use glfw::{Action, Context, Glfw, Modifiers, Key, Window, WindowEvent, WindowMode};
use imgui::{Context as ImContext, MenuItem, im_str};
use imgui_glfw::ImguiGLFW;
use nes::{Button, ButtonState, ControlMessage, EmulationState, FrameBuffer, VideoMessage};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender};

pub struct DebuggerWindow {
    events: Receiver<(f64, WindowEvent)>,
    frame_buffer: Arc<Mutex<FrameBuffer>>,
    glfw: Glfw,
    imgui: ImContext,
    imgui_glfw: ImguiGLFW,
    receive_frame: Receiver<VideoMessage>,
    send_control: Sender<ControlMessage>,
    views: Vec<Box<dyn View>>,
    window: Window,
}

impl DebuggerWindow {
    pub fn new(
        send_control: Sender<ControlMessage>,
        receive_frame: Receiver<VideoMessage>,
        frame_buffer: Arc<Mutex<FrameBuffer>>,
        record: bool,
        arecord: bool,
    ) -> Self {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
        glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));

        let (mut window, events) = glfw.create_window(
            1280,
            720,
            "NES Debugger",
            WindowMode::Windowed,
        ).unwrap();

        window.make_current();
        window.set_all_polling(true);

        gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
        }

        let mut imgui = ImContext::create();
        let imgui_glfw = ImguiGLFW::new(&mut imgui, &mut window);

        let views: Vec<Box<dyn View>> = vec![
            // Box::new(CpuView::new()),
            // Box::new(DebugView::new()),
            Box::new(NesView::new()),
            // Box::new(PpuView::new()),
            // Box::new(LogView::new()),
            // Box::new(PaletteView::new()),
            Box::new(ScreenshotView::new()),
            // Box::new(NametableView::new()),
            // Box::new(PatternView::new()),
            Box::new(RecordView::new(record)),
            Box::new(AudioRecordView::new(arecord)),
            // Box::new(OamView::new()),
        ];

        send_control.send(ControlMessage::SetState(EmulationState::Run)).unwrap();

        Self {
            events,
            frame_buffer,
            glfw,
            imgui,
            imgui_glfw,
            receive_frame,
            send_control,
            views,
            window,
        }
    }

    pub fn run(&mut self) {
        while !self.window.should_close() {

            if let Ok(message) = self.receive_frame.try_recv() {
                match message {
                    VideoMessage::FrameAvailable => {
                        let mut fb = self.frame_buffer.lock().expect("Can't lock framebuffer for reading");
                        self.views.iter_mut().for_each(|v| v.on_frame(&mut fb));
                    }
                }
            }

            unsafe {
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            }

            let ui = self.imgui_glfw.frame(&mut self.window, &mut self.imgui);
            let views = &mut self.views;
            let window = &mut self.window;

            ui.main_menu_bar(|| {
                ui.menu(im_str!("File"), true, || {
                    if MenuItem::new(im_str!("Quit")).shortcut(im_str!("Ctrl-Q")).build(&ui) {
                        window.set_should_close(true);
                    }
                });

                ui.menu(im_str!("Views"), true, || {
                    views.iter_mut().for_each(|v| v.main_menu(&ui));
                });

                views.iter_mut().for_each(|v| v.custom_menu(&ui));
            });

            views.iter_mut().for_each(|v| v.window(&ui));

            self.imgui_glfw.draw(ui, &mut self.window);
            self.window.swap_buffers();

            self.glfw.poll_events();
            for (_, event) in glfw::flush_messages(&self.events) {
                self.imgui_glfw.handle_event(&mut self.imgui, &event);

                views.iter_mut().for_each(|v| v.key_event(&event));

                match event {
                    WindowEvent::Key(Key::Q, _, Action::Press, Modifiers::Control) => {
                        self.window.set_should_close(true);
                    }

                    event => {
                        let nes_button_press = to_nes(&event);

                        if let Some((button, state)) = nes_button_press {
                            let message = ControlMessage::ControllerInput(button, state);
                            self.send_control.send(message).expect("NES thread hung up!");
                        }
                    }
                }
            }
        }

        self.views.iter_mut().for_each(|v| v.destroy());

        self.send_control.send(ControlMessage::SetState(EmulationState::Kill)).unwrap();
    }
}

/// Convert a GLFW key event to an NES controller input event
fn to_nes(event: &WindowEvent) -> Option<(Button, ButtonState)> {
    if let WindowEvent::Key(key, _, action, _) = event {
        let state = match action {
            Action::Press   => Some(ButtonState::Press),
            Action::Release => Some(ButtonState::Release),
            Action::Repeat  => None,
        };

        let button = match key {
            Key::Enter     => Some(Button::Start),
            Key::Backspace => Some(Button::Select),
            Key::Z         => Some(Button::A),
            Key::X         => Some(Button::B),
            Key::Up        => Some(Button::Up),
            Key::Down      => Some(Button::Down),
            Key::Left      => Some(Button::Left),
            Key::Right     => Some(Button::Right),
            _              => None,
        };

        match (button, state) {
            (Some(b), Some(s)) => Some((b, s)),
            _                  => None,
        }
    } else {
        None
    }
}
