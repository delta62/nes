use super::divider::Divider;

pub struct Sweep {
    divider: Divider,
    enabled: bool,
    negate: bool,
    ones_complement: bool,
    shift_count: u8,
}

impl Sweep {
    pub fn new(ones_complement: bool) -> Self {
        Self {
            divider: Divider::new(),
            enabled: false,
            negate: false,
            ones_complement,
            shift_count: 0,
        }
    }

    pub fn update(&mut self, val: u8) {
        self.enabled = bit!(val, 7);
        self.divider.set_period(mask!(val, 0x70) >> 4);
        self.divider.reload();
        self.negate = bit!(val, 3);
        self.shift_count = mask!(val, 0x07);
    }

    pub fn clock(&mut self, period: u16) -> Option<u16> {
        let is_muted = self.mute(period);
        let divider_clock = self.divider.clock();

        if divider_clock && self.enabled && !is_muted {
            Some(self.calc_target(period))
        } else {
            None
        }
    }

    pub fn mute(&self, period: u16) -> bool {
        let target_period = self.calc_target(period);
        period < 8 || target_period > 0x07FF
    }

    fn calc_target(&self, period: u16) -> u16 {
        let period = period as i32;
        let shifted_period = period >> self.shift_count as i32;

        if self.negate {
            let ones_offset = if self.ones_complement { 1 } else { 0 };
            period.wrapping_sub(shifted_period).wrapping_sub(ones_offset) as u16
        } else {
            period.wrapping_add(shifted_period) as u16
        }
    }
}
