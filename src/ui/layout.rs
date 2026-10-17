//! Layout geometry, mirroring the C `tui_recompute_layout` formulas.

/// Minimum supported terminal columns.
pub const MIN_COLS: u16 = 60;

/// Minimum supported terminal rows.
pub const MIN_ROWS: u16 = 22;

/// Computed column/row geometry for the three-panel TUI.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Layout {
    pub cols: u16,
    pub rows: u16,
    pub lx: i32,
    pub lw: i32,
    pub mx: i32,
    pub mw: i32,
    pub rx: i32,
    pub rw: i32,
    pub div1: i32,
    pub div2: i32,
    pub gy: i32,
    pub gh: i32,
    pub gpx: i32,
    pub gpw: i32,
    pub status_y: i32,
    pub result_max_show: i32,
}

impl Layout {
    /// Recompute the layout from the actual terminal size, mirroring the C
    /// `tui_recompute_layout` formulas exactly.
    pub fn compute(cols: u16, rows: u16) -> Self {
        let tw = cols as i32;
        let th = rows as i32;
        let content_w = tw - 4;

        let lw = content_w * 22 / 76;
        let mw = content_w * 32 / 76;
        let rw = content_w - lw - mw;

        let lx = 1;
        let div1 = lx + lw;
        let mx = div1 + 1;
        let div2 = mx + mw;
        let rx = div2 + 1;

        let gy = 4;
        let gh = (th - 14).max(6);
        let gpx = mx + 2;
        let gpw = (mw - 4).max(10);
        let status_y = th - 2;
        let result_max_show = (th - 14).max(1);

        Self {
            cols,
            rows,
            lx,
            lw,
            mx,
            mw,
            rx,
            rw,
            div1,
            div2,
            gy,
            gh,
            gpx,
            gpw,
            status_y,
            result_max_show,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_at_80x24_matches_c() {
        let l = Layout::compute(80, 24);
        assert_eq!(
            [
                l.lx,
                l.lw,
                l.mx,
                l.mw,
                l.rx,
                l.rw,
                l.div1,
                l.div2,
                l.gy,
                l.gh,
                l.gpx,
                l.gpw,
                l.status_y,
                l.result_max_show
            ],
            [1, 22, 24, 32, 57, 22, 23, 56, 4, 10, 26, 28, 22, 10]
        );
    }

    #[test]
    fn layout_at_minimum_60x22_matches_c() {
        let l = Layout::compute(60, 22);
        assert_eq!(
            [
                l.lx,
                l.lw,
                l.mx,
                l.mw,
                l.rx,
                l.rw,
                l.div1,
                l.div2,
                l.gy,
                l.gh,
                l.gpx,
                l.gpw,
                l.status_y,
                l.result_max_show
            ],
            [1, 16, 18, 23, 42, 17, 17, 41, 4, 8, 20, 19, 20, 8]
        );
    }

    #[test]
    fn layout_at_120x40_matches_c() {
        let l = Layout::compute(120, 40);
        assert_eq!(
            [
                l.lx,
                l.lw,
                l.mx,
                l.mw,
                l.rx,
                l.rw,
                l.div1,
                l.div2,
                l.gy,
                l.gh,
                l.gpx,
                l.gpw,
                l.status_y,
                l.result_max_show
            ],
            [1, 33, 35, 48, 84, 35, 34, 83, 4, 26, 37, 44, 38, 26]
        );
    }

    #[test]
    fn layout_floors_apply_on_short_terminals() {
        let l = Layout::compute(60, 18);
        assert_eq!(l.gh, 6);
        assert_eq!(l.result_max_show, 4);
        assert_eq!(l.status_y, 16);
        let l = Layout::compute(40, 22);
        assert_eq!(l.gpw, 11);
    }
}
