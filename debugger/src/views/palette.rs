use super::View;
use super::texture::Texture;
use nes::{Nes, Mem, Rgb};
use crate::macros::press_alt;
use glfw::{Key, WindowEvent};
use imgui::{Image, MenuItem, Ui, Window, im_str};

pub struct PaletteView {
    opened: bool,
    needs_update: bool,
    textures: Vec<Texture>,
}

impl PaletteView {
    pub fn new() -> Self {
        let opened = true;
        let needs_update = false;
        let textures = vec![
            // Background palette
            Texture::new(), Texture::new(), Texture::new(), Texture::new(),
            Texture::new(), Texture::new(), Texture::new(), Texture::new(),
            Texture::new(), Texture::new(), Texture::new(), Texture::new(),
            Texture::new(), Texture::new(), Texture::new(), Texture::new(),

            // Sprite palette
            Texture::new(), Texture::new(), Texture::new(), Texture::new(),
            Texture::new(), Texture::new(), Texture::new(), Texture::new(),
            Texture::new(), Texture::new(), Texture::new(), Texture::new(),
            Texture::new(), Texture::new(), Texture::new(), Texture::new(),
        ];

        Self { needs_update, opened, textures }
    }
}

impl View for PaletteView { }
//     fn main_menu(&mut self, ui: &Ui) {
//         let toggle = MenuItem::new(im_str!("Palette"))
//             .shortcut(im_str!("Alt-A"))
//             .selected(self.opened)
//             .build(&ui);
//
//         if toggle {
//             self.opened = !self.opened;
//         }
//     }
//
//     fn window(&mut self, ui: &Ui, _nes: &Nes, _state: &mut EmulationState) {
//         if !self.opened {
//             return;
//         }
//
//         let textures = &self.textures;
//
//         Window::new(im_str!("Palette"))
//             .opened(&mut self.opened)
//             .content_size([ 275.0, 300.0 ])
//             .always_auto_resize(true)
//             .build(ui, || {
//                 ui.text("Background");
//                 ui.spacing();
//                 ui.spacing();
//                 ui.columns(5, im_str!("palette_background"), false);
//
//                 for i in 0..4 {
//                     ui.text(format!("0x{:04X}", 0x3F00 + i * 4));
//                     ui.next_column();
//
//                     for j in 0..4 {
//                         Image::new(textures[i * 4 + j].id(), [ 24.0, 24.0 ])
//                             .border_col([ 1.0, 1.0, 1.0, 1.0 ])
//                             .build(ui);
//                         ui.next_column();
//                     }
//                 }
//
//                 ui.columns(1, im_str!("palette_reset"), false);
//                 ui.spacing();
//                 ui.spacing();
//                 ui.text("Sprites");
//                 ui.spacing();
//                 ui.spacing();
//                 ui.columns(5, im_str!("palette_sprites"), false);
//
//                 for i in 0..4 {
//                     ui.text(format!("0x{:04X}", 0x3F10 + i * 4));
//                     ui.next_column();
//
//                     for j in 0..4 {
//                         Image::new(textures[i * 4 + 16 + j].id(), [ 24.0, 24.0 ])
//                             .border_col([ 1.0, 1.0, 1.0, 1.0 ])
//                             .build(ui);
//                         ui.next_column();
//                     }
//                 }
//             });
//     }
//
//     fn on_step(&mut self, nes: &Nes) {
//         if !self.needs_update {
//             return;
//         }
//
//         self.needs_update = false;
//
//         let ppu = nes.ppu().borrow();
//         let vram = ppu.vram();
//
//         for x in 0..16 {
//             let addr = 0x3F00 + x;
//             let val = vram.peekb(addr);
//             let rgb = Rgb::from_byte(val);
//
//             self.textures[x as usize].update(1, 1, &[rgb.r, rgb.g, rgb.b]);
//         }
//
//         for x in 0..16 {
//             let addr = 0x3F10 + x;
//             let val = vram.peekb(addr);
//             let rgb = Rgb::from_byte(val);
//
//             self.textures[x as usize + 16].update(1, 1, &[rgb.r, rgb.g, rgb.b]);
//         }
//     }
//
//     fn on_frame(&mut self, _frame: &[u8]) {
//         self.needs_update = true;
//     }
//
//     fn key_event(&mut self, event: &WindowEvent, _state: &mut EmulationState) {
//         if press_alt(Key::A, event) {
//             self.opened = !self.opened;
//         }
//     }
// }
