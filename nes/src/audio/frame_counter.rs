// Cycle counts in the APU are expressed in CPU cycle counts.
// The CPU clocks at twice the speed of the APU, but using APU
// increments results in half step actions

const CLOCK_ONE: u16 = 7_457;
const CLOCK_TWO: u16 = 14_913;
const CLOCK_THREE: u16 = 22_371;
const CLOCK_FOUR: u16 = 29_829;
const CLOCK_FIVE: u16 = 37_281;

enum SequencerMode {
    FourStep,
    FiveStep,
}

pub struct FrameCounterResult {
    pub quarter_frame: bool,
    pub half_frame: bool,
    pub irq: bool,
}

pub struct FrameCounter {
    cycle: u16,
    irq_flag: bool,
    irq_inhibit: bool,
    mode: SequencerMode,
    reset_countdown: u8,
}

impl FrameCounter {
    pub fn new() -> Self {
        Self {
            cycle: 0,
            irq_flag: false,
            irq_inhibit: false,
            mode: SequencerMode::FourStep,
            reset_countdown: 0,
        }
    }

    pub fn step(&mut self) -> FrameCounterResult {
        let mut clock_quarter_half = false;

        if self.reset_countdown == 1 {
            self.cycle = 0;

            if let SequencerMode::FiveStep = self.mode {
                clock_quarter_half = true;
            }
        } else {
            self.cycle += 1;
        }

        if self.reset_countdown > 0 {
            self.reset_countdown -= 1;
        }

        match self.mode {
            SequencerMode::FourStep => self.step4(),
            SequencerMode::FiveStep => self.step5(clock_quarter_half),
        }
    }

    fn step4(&mut self) -> FrameCounterResult {
        self.cycle %= CLOCK_FOUR + 1;

        if self.cycle == CLOCK_FOUR && !self.irq_inhibit {
            self.irq_flag = true;
        }

        let irq = self.irq_flag;

        match self.cycle {
            CLOCK_ONE => FrameCounterResult {
                quarter_frame: true,
                half_frame: false,
                irq,
            },
            CLOCK_TWO => FrameCounterResult {
                quarter_frame: true,
                half_frame: true,
                irq,
            },
            CLOCK_THREE => FrameCounterResult {
                quarter_frame: true,
                half_frame: false,
                irq,
            },
            CLOCK_FOUR => FrameCounterResult {
                quarter_frame: true,
                half_frame: true,
                irq,
            },
            _ => FrameCounterResult {
                quarter_frame: false,
                half_frame: false,
                irq,
            },
        }
    }

    fn step5(&mut self, clock_quarter_half: bool) -> FrameCounterResult {
        self.cycle %= CLOCK_FIVE + 1;

        let irq = self.irq_flag;

        match self.cycle {
            CLOCK_ONE => FrameCounterResult {
                quarter_frame: true,
                half_frame: clock_quarter_half,
                irq,
            },
            CLOCK_TWO => FrameCounterResult {
                quarter_frame: true,
                half_frame: true,
                irq,
            },
            CLOCK_THREE => FrameCounterResult {
                quarter_frame: true,
                half_frame: clock_quarter_half,
                irq,
            },
            CLOCK_FIVE => FrameCounterResult {
                quarter_frame: true,
                half_frame: true,
                irq,
            },
            _ => FrameCounterResult {
                quarter_frame: clock_quarter_half,
                half_frame: clock_quarter_half,
                irq,
            },
        }
    }

    pub fn update(&mut self, val: u8) {
        self.mode = if bit!(val, 7) {
            SequencerMode::FiveStep
        } else {
            SequencerMode::FourStep
        };

        self.irq_inhibit = bit!(val, 6);

        if self.irq_inhibit {
            self.irq_flag = false;
        }

        if self.cycle % 2 == 0 {
            self.reset_countdown = 3;
        } else {
            self.reset_countdown = 4;
        }
    }

    pub fn reset_irq(&mut self) {
        self.irq_flag = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resets_irq() {
        let mut fc = FrameCounter::new();
        fc.irq_flag = true;
        fc.reset_irq();

        assert!(!fc.irq_flag);
    }

    #[test]
    fn update_resets_irq_when_inhibited() {
        let mut fc = FrameCounter::new();
        fc.irq_flag = true;
        fc.update(0x40);

        assert!(!fc.irq_flag);
    }

    #[test]
    fn update_resets_timer_after_3_cycles() {
        let base_cycle = 32;
        let mut fc = FrameCounter::new();
        fc.cycle = base_cycle;
        fc.update(0x00);

        for i in 0..3 {
            assert_eq!(fc.cycle, base_cycle + i);
            fc.step();
        }

        assert_eq!(fc.cycle, 0);
    }

    #[test]
    fn update_clocks_querter_half_after_3_cycles() {
        let base_cycle = 32;
        let mut fc = FrameCounter::new();
        fc.cycle = base_cycle;
        fc.update(0x80);

        for _ in 0..2 {
            let FrameCounterResult {
                quarter_frame,
                half_frame,
                ..
            } = fc.step();
            assert!(!quarter_frame);
            assert!(!half_frame);
        }

        let FrameCounterResult {
            quarter_frame,
            half_frame,
            ..
        } = fc.step();
        assert!(quarter_frame);
        assert!(half_frame);
    }

    #[test]
    fn update_resets_timer_after_4_cycles() {
        let base_cycle = 33;
        let mut fc = FrameCounter::new();
        fc.cycle = base_cycle;
        fc.update(0x00);

        for i in 0..4 {
            assert_eq!(fc.cycle, base_cycle + i);
            fc.step();
        }

        assert_eq!(fc.cycle, 0);
    }

    #[test]
    fn update_clocks_querter_half_after_4_cycles() {
        let base_cycle = 33;
        let mut fc = FrameCounter::new();
        fc.cycle = base_cycle;
        fc.update(0x80);

        for _ in 0..3 {
            let FrameCounterResult {
                quarter_frame,
                half_frame,
                ..
            } = fc.step();
            assert!(!quarter_frame);
            assert!(!half_frame);
        }

        let FrameCounterResult {
            quarter_frame,
            half_frame,
            ..
        } = fc.step();
        assert!(quarter_frame);
        assert!(half_frame);
    }

    mod step4 {
        use super::*;

        #[test]
        fn emits_irqs() {
            let mut fc = FrameCounter::new();
            fc.cycle = CLOCK_FOUR - 1;

            let res = fc.step();
            assert!(res.irq);
        }

        #[test]
        fn inhibits_irqs() {
            let mut fc = FrameCounter::new();
            fc.cycle = CLOCK_FOUR - 1;
            fc.irq_inhibit = true;

            let res = fc.step();
            assert!(!res.irq);
        }

        #[test]
        fn emits_quarter_steps() {
            let mut fc = FrameCounter::new();
            let mut steps = Vec::with_capacity(4);

            for i in 0..CLOCK_FOUR {
                let result = fc.step();
                if result.quarter_frame {
                    steps.push(i + 1);
                }
            }

            assert_eq!(steps, vec![CLOCK_ONE, CLOCK_TWO, CLOCK_THREE, CLOCK_FOUR]);
        }

        #[test]
        fn emits_half_steps() {
            let mut fc = FrameCounter::new();
            let mut steps = Vec::with_capacity(2);

            for i in 0..CLOCK_FOUR {
                let result = fc.step();
                if result.half_frame {
                    steps.push(i + 1);
                }
            }

            assert_eq!(steps, vec![CLOCK_TWO, CLOCK_FOUR]);
        }

        #[test]
        fn sequencer_loops() {
            let mut fc = FrameCounter::new();
            fc.cycle = CLOCK_FOUR;
            fc.step();

            assert_eq!(fc.cycle, 0);
        }
    }

    mod step5 {
        use super::*;

        #[test]
        fn emits_quarter_steps() {
            let mut fc = FrameCounter::new();
            fc.mode = SequencerMode::FiveStep;
            let mut steps = Vec::with_capacity(4);

            for i in 0..CLOCK_FIVE {
                let result = fc.step();
                if result.quarter_frame {
                    steps.push(i + 1);
                }
            }

            assert_eq!(steps, vec![CLOCK_ONE, CLOCK_TWO, CLOCK_THREE, CLOCK_FIVE]);
        }

        #[test]
        fn emits_half_steps() {
            let mut fc = FrameCounter::new();
            fc.mode = SequencerMode::FiveStep;
            let mut steps = Vec::with_capacity(2);

            for i in 0..CLOCK_FIVE {
                let result = fc.step();
                if result.half_frame {
                    steps.push(i + 1);
                }
            }

            assert_eq!(steps, vec![CLOCK_TWO, CLOCK_FIVE]);
        }

        #[test]
        fn sequencer_loops() {
            let mut fc = FrameCounter::new();
            fc.mode = SequencerMode::FiveStep;
            fc.cycle = CLOCK_FIVE;
            fc.step();

            assert_eq!(fc.cycle, 0);
        }
    }
}
