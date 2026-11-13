//! `Trig.SINE` / `Trig.COSINE`: 16384 steps to a full turn, scaled by 16384, truncated.

/// One step of the table (2π / 16384).
const STEP: f64 = 3.834951969714103E-4;

pub fn sine(i: i32) -> i32 {
    (16384.0 * (i as f64 * STEP).sin()) as i32
}

pub fn cosine(i: i32) -> i32 {
    (16384.0 * (i as f64 * STEP).cos()) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_quarter_turn_is_4096_steps() {
        assert_eq!((sine(0), cosine(0), sine(4096)), (0, 16384, 16384));
    }
}
