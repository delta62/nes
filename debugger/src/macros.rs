macro_rules! shortcut {
    ($key:ident) => {
        ::egui::KeyboardShortcut {
            modifiers: ::egui::Modifiers::NONE,
            logical_key: ::egui::Key::$key,
        }
    };

    ($mod:ident, $key:ident) => {
        ::egui::KeyboardShortcut {
            modifiers: ::egui::Modifiers::$mod,
            logical_key: ::egui::Key::$key,
        }
    };
}
