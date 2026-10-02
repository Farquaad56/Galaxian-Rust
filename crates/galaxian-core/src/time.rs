//! Time types and hardware timing constants (Partie F, T0.1.2).
//!
//! The unit of time in the simulator is the Z80 cycle (T-state) — rule B.4:
//! time is the central data. All values derive from KB-02 (clocks and timing).

/// Absolute time in Z80 cycles (T-states). // KB-02 §derived values; rule B.4
pub type Cycles = u64;

/// Z80 clock frequency: XTAL/6 = 3.072 MHz. // KB-02 §clocks
pub const Z80_HZ: u64 = 3_072_000;

/// HSYNC frequency: XTAL/3/192/2 = 16 kHz (one line per period). // KB-02 §clocks
pub const HSYNC_HZ: u64 = 16_000;

/// Z80 cycles per video line: `Z80_HZ` / `HSYNC_HZ` = `3_072_000` / `16_000`. // KB-02 §derived values
pub const CYCLES_PER_LINE: Cycles = 192;

/// Lines per frame (`GALAXIAN_VTOTAL`). // KB-02 §MAME constants
pub const LINES_PER_FRAME: u64 = 264;

/// Z80 cycles per frame: `CYCLES_PER_LINE` * `LINES_PER_FRAME`. // KB-02 §derived values
pub const CYCLES_PER_FRAME: Cycles = 50_688;

/// Pixels per line at the pixel clock XTAL/3 (one HSYNC period). // KB-02 §clocks
pub const PIXELS_PER_LINE: u64 = 384;

/// The pixel clock (XTAL/3 = 6.144 MHz) runs at twice the Z80 clock
/// (3.072 MHz): one Z80 cycle spans two consecutive pixels ("pixel = 2 × cycle"). // KB-02 §clocks
pub const PIXELS_PER_CYCLE: u64 = 2;

/// Line number of an absolute cycle count (line 0 starts at cycle 0).
#[must_use]
pub fn cycles_to_line(cycles: Cycles) -> u64 {
    cycles / CYCLES_PER_LINE
}

/// First cycle of the given line.
#[must_use]
pub fn line_to_cycles(line: u64) -> Cycles {
    line * CYCLES_PER_LINE
}

/// Convert an absolute cycle count to (line, pixel column).
///
/// A Z80 cycle spans two consecutive pixels; the returned column is the first
/// of them, so it is always even and in `0..=PIXELS_PER_LINE - 2` (i.e. up to
/// 382: the last cycle of a line covers pixels 382-383).
#[must_use]
pub fn cycles_to_line_col(cycles: Cycles) -> (u64, u64) {
    let line = cycles / CYCLES_PER_LINE;
    let col = (cycles % CYCLES_PER_LINE) * PIXELS_PER_CYCLE;
    (line, col)
}

/// Convert (line, pixel column) to the first cycle that covers it.
///
/// Two pixels share one Z80 cycle, so an odd column maps to the same cycle as
/// its preceding even column. A `col` beyond `PIXELS_PER_LINE - 1` carries into
/// a later cycle (e.g. `(0, 384)` -> cycle 192); callers passing in-range
/// columns get the exact covering cycle.
#[must_use]
pub fn line_col_to_cycles(line: u64, col: u64) -> Cycles {
    line * CYCLES_PER_LINE + col / PIXELS_PER_CYCLE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycles_per_line_times_lines_per_frame_is_cycles_per_frame() {
        assert_eq!(CYCLES_PER_LINE * LINES_PER_FRAME, CYCLES_PER_FRAME);
        assert_eq!(CYCLES_PER_FRAME, 50_688);
    }

    #[test]
    fn line_240_starts_at_cycle_46_080() {
        // VBSTART = 240 (KB-02 §MAME constants): entry into VBLANK.
        assert_eq!(line_to_cycles(240), 46_080);
        assert_eq!(cycles_to_line_col(46_080), (240, 0));
    }

    #[test]
    fn cycles_per_line_is_z80_over_hsync() {
        assert_eq!(Z80_HZ / HSYNC_HZ, CYCLES_PER_LINE);
    }

    #[test]
    fn pixel_clock_runs_at_twice_the_z80_clock() {
        // 6.144 MHz / 3.072 MHz = 2 pixels per Z80 cycle (KB-02 §clocks).
        assert_eq!(PIXELS_PER_LINE / CYCLES_PER_LINE, PIXELS_PER_CYCLE);
        assert_eq!(PIXELS_PER_CYCLE, 2);
    }

    #[test]
    fn roundtrip_cycle_to_line_col_and_back() {
        for c in [
            0u64,
            1,
            191,
            192,
            46_079,
            46_080,
            CYCLES_PER_FRAME - 1,
            CYCLES_PER_FRAME,
        ] {
            let (line, col) = cycles_to_line_col(c);
            assert_eq!(line_col_to_cycles(line, col), c);
        }
    }

    #[test]
    fn odd_pixel_shares_cycle_with_preceding_even() {
        // Cycle 193 is the second cycle of line 1 and covers pixels 2 and 3.
        assert_eq!(cycles_to_line_col(193), (1, 2));
        assert_eq!(line_col_to_cycles(1, 3), 193);
    }

    #[test]
    fn frame_boundary_is_next_frame_line_zero() {
        // Absolute time continues past the frame: cycle 50_688 is line 264.
        assert_eq!(cycles_to_line(CYCLES_PER_FRAME), LINES_PER_FRAME);
        assert_eq!(cycles_to_line_col(CYCLES_PER_FRAME), (LINES_PER_FRAME, 0));
    }

    #[test]
    fn visible_region_boundaries() {
        // KB-02 §derived values: visible lines 16..239, pixels 0..255.
        assert_eq!(line_to_cycles(16), 3_072);
        assert_eq!(cycles_to_line_col(line_to_cycles(240)), (240, 0)); // VBSTART
    }
}
