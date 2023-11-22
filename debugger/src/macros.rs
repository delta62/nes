// pub fn press(key: Key, event: &WindowEvent) -> bool {
//     let mods = Modifiers::empty();
//     match *event {
//         WindowEvent::Key(k, _, Action::Press, m) if k == key && m == mods=> true,
//         _ => false,
//     }
// }

// pub fn press_alt(key: Key, event: &WindowEvent) -> bool {
//     match *event {
//         WindowEvent::Key(k, _, Action::Press, Modifiers::Alt) if k == key => true,
//         _ => false,
//     }
// }

// pub fn press_ctrl(key: Key, event: &WindowEvent) -> bool {
//     match *event {
//         WindowEvent::Key(k, _, Action::Press, Modifiers::Control) if k == key => true,
//         _ => false,
//     }
// }
