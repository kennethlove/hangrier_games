//! Hex-grid math for the arena map.
//!
//! v1 layout is a flat 7-tile pointy-top hex cluster: Cornucopia in the
//! center surrounded by 6 sectors numbered 1..6 clockwise starting at the
//! top-right.
//!
//! Coordinate system is *axial* (`q`, `r`); see
//! https://www.redblobgames.com/grids/hexagons/ for the full reference.
//!
//! Game logic does not depend on the pixel layout — only on neighbor
//! adjacency and area identity. The pixel-layout half (`to_pixel`) is here
//! so the frontend can render without re-deriving the math.

use crate::areas::Area;

/// Axial hex coordinate (`q`, `r`). Pointy-top orientation.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Axial {
    pub q: i32,
    pub r: i32,
}

impl Axial {
    pub const fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    /// The six neighbor coords in pointy-top axial order, starting at the
    /// **top-right** neighbor and going clockwise. Order matches the
    /// numbering of [`default_layout`].
    pub fn neighbors(self) -> [Axial; 6] {
        // (Δq, Δr) for pointy-top, clockwise from top-right.
        const DIRS: [(i32, i32); 6] = [
            (1, -1), // top-right (Sector1)
            (1, 0),  // right     (Sector2)
            (0, 1),  // bot-right (Sector3)
            (-1, 1), // bot-left  (Sector4)
            (-1, 0), // left      (Sector5)
            (0, -1), // top-left  (Sector6)
        ];
        let mut out = [Axial::new(0, 0); 6];
        for (i, (dq, dr)) in DIRS.iter().enumerate() {
            out[i] = Axial::new(self.q + dq, self.r + dr);
        }
        out
    }

    /// Hex distance: `(|q1 - q2| + |q1 + r1 - q2 - r2| + |r1 - r2|) / 2`.
    pub fn distance(self, other: Axial) -> i32 {
        let dq = (self.q - other.q).abs();
        let dr = (self.r - other.r).abs();
        let ds = (self.q + self.r - other.q - other.r).abs();
        (dq + dr + ds) / 2
    }

    /// Convert to pixel coords (pointy-top). `size` is the hex
    /// center-to-corner radius. Returned coords are centered at the origin.
    pub fn to_pixel(self, size: f64) -> (f64, f64) {
        let q = self.q as f64;
        let r = self.r as f64;
        let x = size * (3.0_f64.sqrt() * q + 3.0_f64.sqrt() / 2.0 * r);
        let y = size * (3.0 / 2.0 * r);
        (x, y)
    }
}

/// The 7-tile default layout, ordered to match the area's numeric label
/// (`0` for Cornucopia, `1..6` for the surrounding sectors clockwise from
/// top-right). The order in this slice **is** the numeric label.
pub fn default_layout() -> [(Area, Axial); 7] {
    [
        (Area::Cornucopia, Axial::new(0, 0)),
        (Area::Sector1, Axial::new(1, -1)),
        (Area::Sector2, Axial::new(1, 0)),
        (Area::Sector3, Axial::new(0, 1)),
        (Area::Sector4, Axial::new(-1, 1)),
        (Area::Sector5, Axial::new(-1, 0)),
        (Area::Sector6, Axial::new(0, -1)),
    ]
}

/// Area-local sub-tile coordinate. Each area-hex is subdivided into 7
/// sub-hexes (1 center + 6 ring) using the same axial system as the
/// top-level layout. Sub-tiles are presentation/positioning only — game
/// logic still operates at the area level.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SubAxial {
    pub q: i32,
    pub r: i32,
}

impl SubAxial {
    pub const fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    /// Convert to pixel coords relative to the parent hex center, using
    /// the same pointy-top math as `Axial::to_pixel`. `sub_size` is the
    /// sub-hex center-to-corner radius.
    pub fn to_pixel(self, sub_size: f64) -> (f64, f64) {
        let q = self.q as f64;
        let r = self.r as f64;
        let x = sub_size * (3.0_f64.sqrt() * q + 3.0_f64.sqrt() / 2.0 * r);
        let y = sub_size * (3.0 / 2.0 * r);
        (x, y)
    }
}

/// The 7 sub-tile slots within a single area-hex, in the same numeric
/// order as `default_layout()`: index 0 = center, 1..6 = ring clockwise
/// from top-right.
pub const SUB_SLOTS: [SubAxial; 7] = [
    SubAxial::new(0, 0),
    SubAxial::new(1, -1),
    SubAxial::new(1, 0),
    SubAxial::new(0, 1),
    SubAxial::new(-1, 1),
    SubAxial::new(-1, 0),
    SubAxial::new(0, -1),
];

/// Sub-hex size relative to the parent area-hex size. The 7-cluster
/// layout (1 center + 6 ring) fits inside the parent hex when each
/// sub-hex has center-to-corner radius `parent_size / 3` — the ring
/// sub-hex centers sit at distance `parent_size * sqrt(3) / 3` from the
/// center, with their outer edges grazing the parent's inscribed circle.
pub const SUB_SIZE_RATIO: f64 = 1.0 / 3.0;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn neighbors_returns_six_unique_coords() {
        let n = Axial::new(0, 0).neighbors();
        let set: HashSet<Axial> = n.iter().copied().collect();
        assert_eq!(set.len(), 6, "got duplicates: {:?}", n);
    }

    #[test]
    fn neighbors_clockwise_from_top_right() {
        let n = Axial::new(0, 0).neighbors();
        // Pointy-top axial: top-right is (1, -1), right is (1, 0), etc.
        assert_eq!(n[0], Axial::new(1, -1));
        assert_eq!(n[1], Axial::new(1, 0));
        assert_eq!(n[2], Axial::new(0, 1));
        assert_eq!(n[3], Axial::new(-1, 1));
        assert_eq!(n[4], Axial::new(-1, 0));
        assert_eq!(n[5], Axial::new(0, -1));
    }

    #[test]
    fn distance_to_self_is_zero() {
        let a = Axial::new(2, -1);
        assert_eq!(a.distance(a), 0);
    }

    #[test]
    fn distance_cornucopia_to_each_sector_is_one() {
        let center = Axial::new(0, 0);
        for s in default_layout().iter().skip(1) {
            assert_eq!(center.distance(s.1), 1, "sector {:?}", s.0);
        }
    }

    #[test]
    fn distance_opposite_sectors_is_two() {
        // Sector1 (top-right) ↔ Sector4 (bottom-left)
        assert_eq!(
            Axial::new(1, -1).distance(Axial::new(-1, 1)),
            2,
            "Sector1 ↔ Sector4 should be 2"
        );
        // Sector2 ↔ Sector5
        assert_eq!(Axial::new(1, 0).distance(Axial::new(-1, 0)), 2);
        // Sector3 ↔ Sector6
        assert_eq!(Axial::new(0, 1).distance(Axial::new(0, -1)), 2);
    }

    #[test]
    fn distance_adjacent_sectors_is_one() {
        // Sector1 ↔ Sector2 (top-right ↔ right)
        assert_eq!(Axial::new(1, -1).distance(Axial::new(1, 0)), 1);
        // Sector2 ↔ Sector3
        assert_eq!(Axial::new(1, 0).distance(Axial::new(0, 1)), 1);
        // Wrap: Sector6 ↔ Sector1
        assert_eq!(Axial::new(0, -1).distance(Axial::new(1, -1)), 1);
    }

    #[test]
    fn default_layout_has_seven_unique_axials() {
        let layout = default_layout();
        let coords: HashSet<Axial> = layout.iter().map(|(_, ax)| *ax).collect();
        assert_eq!(coords.len(), 7);
    }

    #[test]
    fn default_layout_first_is_cornucopia() {
        assert_eq!(default_layout()[0].0, Area::Cornucopia);
        assert_eq!(default_layout()[0].1, Axial::new(0, 0));
    }

    #[test]
    fn to_pixel_origin_for_center() {
        let (x, y) = Axial::new(0, 0).to_pixel(60.0);
        assert!(x.abs() < 1e-9, "x={x}");
        assert!(y.abs() < 1e-9, "y={y}");
    }

    #[test]
    fn pixel_distances_between_layout_neighbors_are_consistent() {
        let layout = default_layout();
        let size = 60.0_f64;
        let center_px = Axial::new(0, 0).to_pixel(size);
        // Each surrounding sector should be the same pixel distance from
        // the center: 2 * size * cos(30°) = size * sqrt(3) for pointy-top
        // edge-to-edge — wait, that's edge midpoints. Center-to-center
        // distance for adjacent hexes is also size * sqrt(3) for pointy-top.
        let expected = size * 3.0_f64.sqrt();
        for (_area, ax) in layout.iter().skip(1) {
            let (x, y) = ax.to_pixel(size);
            let dx = x - center_px.0;
            let dy = y - center_px.1;
            let d = (dx * dx + dy * dy).sqrt();
            assert!(
                (d - expected).abs() < 1e-6,
                "{:?} center-distance {} != {}",
                ax,
                d,
                expected
            );
        }
    }

    #[test]
    fn sub_slots_has_seven_unique_coords() {
        let set: HashSet<SubAxial> = SUB_SLOTS.iter().copied().collect();
        assert_eq!(set.len(), 7);
    }

    #[test]
    fn sub_slots_first_is_center() {
        assert_eq!(SUB_SLOTS[0], SubAxial::new(0, 0));
    }

    #[test]
    fn sub_slots_no_overlap_at_default_ratio() {
        // Sub-hexes are non-overlapping iff center-to-center distance >=
        // 2 * sub_size * cos(30°) = sub_size * sqrt(3) for pointy-top
        // adjacent hexes. With SUB_SIZE_RATIO = 1/3 and parent_size = 90,
        // sub_size = 30, expected min center distance = 30 * sqrt(3).
        let parent_size = 90.0_f64;
        let sub_size = parent_size * SUB_SIZE_RATIO;
        let min_dist = sub_size * 3.0_f64.sqrt();
        for (i, a) in SUB_SLOTS.iter().enumerate() {
            for b in SUB_SLOTS.iter().skip(i + 1) {
                let (ax, ay) = a.to_pixel(sub_size);
                let (bx, by) = b.to_pixel(sub_size);
                let d = ((ax - bx).powi(2) + (ay - by).powi(2)).sqrt();
                assert!(
                    d + 1e-6 >= min_dist,
                    "sub-slots {:?} and {:?} overlap (d={}, min={})",
                    a,
                    b,
                    d,
                    min_dist,
                );
            }
        }
    }
}
