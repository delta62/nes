use crate::macros::press_alt;
use glfw::{Key, WindowEvent};
use imgui::{MenuItem, TreeNode, Ui, Window, im_str};
use nes::{Mem, Nes};
use super::View;

pub struct PpuView {
    opened: bool,
}

impl PpuView {
    pub fn new() -> Self {
        let opened = false;
        Self { opened }
    }
}

impl View for PpuView {}
//     fn main_menu(&mut self, ui: &Ui) {
//         let toggle = MenuItem::new(im_str!("PPU"))
//             .shortcut(im_str!("Alt-P"))
//             .selected(self.opened)
//             .build(&ui);
//
//         if toggle {
//             self.opened = !self.opened;
//         }
//     }
//
//     fn window(&mut self, ui: &Ui, nes: &Nes, _state: &mut EmulationState) {
//         if !self.opened {
//             return;
//         }
//
//         let ppu = nes.ppu().borrow();
//
//         Window::new(im_str!("PPU"))
//             .opened(&mut self.opened)
//             .size_constraints([ 240.0, 200.0 ], [ f32::MAX, f32::MAX ])
//             .build(&ui, || {
//                 ui.label_text(im_str!("Cycle"),     &im_str!("{}", ppu.cycle()));
//                 ui.label_text(im_str!("Frame"),     &im_str!("{}", ppu.frame()));
//                 ui.label_text(im_str!("Scanline"),  &im_str!("{}", ppu.scanline()));
//                 ui.label_text(im_str!("Pixel"),     &im_str!("{}", ppu.pixel()));
//
//                 ui.spacing();
//                 ui.separator();
//                 ui.spacing();
//
//                 ui.label_text(im_str!("PPUCTRL"),   &im_str!("0x{:02X}", ppu.peekb(0x2000)));
//                 ui.label_text(im_str!("PPUMASK"),   &im_str!("0x{:02X}", ppu.peekb(0x2001)));
//                 ui.label_text(im_str!("PPUSTATUS"), &im_str!("0x{:02X}", ppu.peekb(0x2002)));
//                 ui.label_text(im_str!("OAMADDR"),   &im_str!("0x{:02X}", ppu.peekb(0x2003)));
//                 ui.label_text(im_str!("OAMDATA"),   &im_str!("0x{:02X}", ppu.peekb(0x2004)));
//                 ui.label_text(im_str!("PPUSCROLL"), &im_str!("0x{:02X}", ppu.peekb(0x2005)));
//                 ui.label_text(im_str!("PPUADDR"),   &im_str!("0x{:02X}", ppu.peekb(0x2006)));
//                 ui.label_text(im_str!("PPUDATA"),   &im_str!("0x{:02X}", ppu.peekb(0x2007)));
//                 ui.label_text(im_str!("OAMDMA"),    &im_str!("0x{:02X}", ppu.peekb(0x2008)));
//
//                 ui.spacing();
//                 ui.separator();
//                 ui.spacing();
//
//                 ui.push_item_width(ui.window_size()[0] * 0.3);
//
//                 TreeNode::new(im_str!("PPUCTRL")).build(&ui, || {
//                     let ctrl = ppu.ppuctrl();
//                     ui.label_text(im_str!("Base NT addr"), &im_str!("0x{:04X}", ctrl.base_nametable_addr()));
//                     ui.label_text(im_str!("VRAM inc"),     &im_str!("{}", ctrl.vram_addr_increment()));
//                     ui.label_text(im_str!("Sprite addr"),  &im_str!("{}", ctrl.sprite_pattern_table_address()));
//                     ui.label_text(im_str!("Backgr addr"),  &im_str!("{}", ctrl.background_pattern_table_address()));
//                     ui.label_text(im_str!("Sprite size"),  &im_str!("8x{}", ctrl.sprite_size()));
//                     ui.label_text(im_str!("Pri/sec sel"),  &im_str!("{}", if ctrl.is_primary() { "primary" } else { "secondary" }));
//                     ui.label_text(im_str!("NMI enabled"),  &im_str!("{}", ctrl.generate_nmi()));
//                 });
//
//                 TreeNode::new(im_str!("PPUMASK")).build(&ui, || {
//                     let mask = ppu.ppumask();
//                     ui.label_text(im_str!("Greyscale"),    &im_str!("{}", mask.greyscale_enabled()));
//                     ui.label_text(im_str!("hide bg 0"),    &im_str!("{}", mask.hide_first_bg_tile()));
//                     ui.label_text(im_str!("hide fg 0"),    &im_str!("{}", mask.hide_first_sprite_tile()));
//                     ui.label_text(im_str!("BG enabled"),   &im_str!("{}", mask.background_rendering_enabled()));
//                     ui.label_text(im_str!("FG enabled"),   &im_str!("{}", mask.sprite_rendering_enabled()));
//                     ui.label_text(im_str!("Emph. red"),    &im_str!("{}", mask.emphasize_red()));
//                     ui.label_text(im_str!("Emph. green"),  &im_str!("{}", mask.emphasize_green()));
//                     ui.label_text(im_str!("Emph. blue"),   &im_str!("{}", mask.emphasize_blue()));
//                 });
//
//                 TreeNode::new(im_str!("PPUSTATUS")).build(&ui, || {
//                     let status = ppu.ppustatus();
//                     ui.label_text(im_str!("Sprite ovflw"), &im_str!("{}", status.sprite_overflow()));
//                     ui.label_text(im_str!("Sprite 0 hit"), &im_str!("{}", status.sprite_zero_hit()));
//                     ui.label_text(im_str!("Vblank start"), &im_str!("{}", status.vblank_started()));
//                 });
//
//                 // TreeNode::new(im_str!("PPUSCROLL")).build(&ui, || {
//                 //     let scroll = ppu.ppuscroll();
//                 //     ui.label_text(im_str!("Scroll X"),     &im_str!("0x{0:02X}", scroll.x()));
//                 //     ui.label_text(im_str!("Scroll Y"),     &im_str!("0x{0:02X}", scroll.y()));
//                 // });
//             });
//     }
//
//     fn key_event(&mut self, event: &WindowEvent, _state: &mut EmulationState) {
//         if press_alt(Key::P, event) {
//             self.opened = !self.opened;
//         }
//     }
// }
