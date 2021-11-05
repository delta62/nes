pub trait Channel {
    fn clock(&mut self);

    fn get(&self) -> u8;

    fn is_running(&self) -> bool;

    fn set_enabled(&mut self, enabled: bool);

    fn half_frame_clock(&mut self);

    fn quarter_frame_clock(&mut self);
}
