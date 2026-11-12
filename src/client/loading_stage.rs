//! The start-up stages and the text and percentage shown for them.
//! Port of `LoadingStage` and the bookkeeping in `Preference_Sub20.method12808`.

use std::time::Instant;

const CHECKING: &str = "Checking for updates - ";
const FETCHING: &str = "Fetching Updates - ";

pub struct Stage {
    /// Text while in this stage, and the text used for the moment it is left.
    pub text: &'static str,
    pub leaving_text: &'static str,
    pub min: i32,
    pub max: i32,
    /// The bar moves from `min` to `max` as the stage reports progress.
    pub interpolates: bool,
}

const fn stage(text: &'static str, leaving_text: &'static str, min: i32, max: i32, interpolates: bool) -> Stage {
    Stage { text, leaving_text, min, max, interpolates }
}

/// The 18 stages in order (`LoadingStage.method6676`); the position is the stage number.
pub const STAGES: [Stage; 18] = [
    stage(CHECKING, CHECKING, 2, 2, false),
    stage(CHECKING, CHECKING, 2, 3, false),
    stage(CHECKING, CHECKING, 3, 3, false),
    stage(CHECKING, CHECKING, 3, 4, false),
    stage(CHECKING, CHECKING, 4, 4, false),
    stage(CHECKING, CHECKING, 4, 5, false),
    stage(CHECKING, CHECKING, 5, 98, true),
    stage(CHECKING, CHECKING, 99, 99, false),
    stage(CHECKING, CHECKING, 100, 100, false),
    stage(FETCHING, FETCHING, 0, 92, true),
    stage(FETCHING, FETCHING, 92, 93, false),
    stage(FETCHING, FETCHING, 94, 95, false),
    stage(FETCHING, FETCHING, 96, 97, false),
    stage(FETCHING, FETCHING, 97, 97, false),
    stage(FETCHING, FETCHING, 97, 97, false),
    stage(FETCHING, FETCHING, 100, 100, false),
    stage(FETCHING, FETCHING, 100, 100, false),
    stage(FETCHING, FETCHING, 100, 100, false),
];

/// The last stage: reaching it means the login screen comes next.
pub const DONE: usize = 17;

/// What the loading screen shows right now.
#[derive(Clone, Debug)]
pub struct Status {
    pub stage: usize,
    pub percent: i32,
    pub text: String,
    /// When the bar's percentage last had to start moving (`Class302.aLong3562`).
    pub started: Instant,
}

impl Status {
    pub fn new() -> Self {
        let s = &STAGES[0];
        Status { stage: 0, percent: 0, text: format!("{}{}%", s.text, 0), started: Instant::now() }
    }

    /// Moves to `next` (which may be the same stage) with the stage's own progress 0..=100.
    pub fn update(&mut self, next: usize, progress: i32) {
        if next == self.stage {
            let s = &STAGES[next];
            if s.interpolates {
                self.percent = progress * (s.max - s.min) / 100 + s.min;
            }
            self.text = format!("{}{}%", s.text, self.percent);
        } else {
            let (old, new) = (&STAGES[self.stage], &STAGES[next]);
            self.percent = old.max;
            self.text = format!("{}{}%", old.leaving_text, old.max);
            if old.interpolates || new.interpolates {
                self.started = Instant::now();
            }
            self.stage = next;
        }
    }

    /// Port of `Class306.method5423`: where the bar is heading next.
    pub fn target(&self) -> i32 {
        let s = &STAGES[self.stage];
        if s.interpolates && self.percent < s.max {
            self.percent + 1
        } else if self.stage < STAGES.len() - 1 {
            if s.min == self.percent {
                s.max
            } else {
                s.min
            }
        } else {
            100
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolating_stage_and_leaving_text() {
        let mut s = Status::new();
        s.update(6, 0);
        assert_eq!((s.percent, s.text.as_str()), (2, "Checking for updates - 2%"));
        s.update(6, 50);
        assert_eq!((s.percent, s.text.as_str()), (51, "Checking for updates - 51%"));
        s.update(9, 0);
        assert_eq!((s.percent, s.text.as_str()), (98, "Checking for updates - 98%"));
        s.update(9, 100);
        assert_eq!(s.text, "Fetching Updates - 92%");
    }
}
