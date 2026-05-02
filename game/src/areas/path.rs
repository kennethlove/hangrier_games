//! Area-level pathfinding: builds a graph over the v1 7-area hex topology
//! and exposes a path-planning helper for tributes.
//!
//! Edge cost is a composite per the design decision (8pq Q2):
//! `stamina_cost + harshness_penalty + closed_penalty`.
//! - `stamina_cost`: per-tribute, per-terrain via `calculate_stamina_cost`
//! - `harshness_penalty`: 0/10/20 for Mild/Moderate/Harsh
//! - `closed_penalty`: high additive cost (`CLOSED_PENALTY`) so closed
//!   areas are routed around when alternatives exist, but still
//!   traversable as a last resort (8pq Q4 = K, "high penalty").

use crate::areas::{Area, AreaDetails};
use crate::pathfinding::{Graph, astar};
use crate::terrain::Harshness;
use crate::tributes::actions::Action;
use crate::tributes::{Tribute, calculate_stamina_cost};
use std::collections::HashMap;
use strum::IntoEnumIterator;

/// Penalty added to any edge entering a closed area. Picked so that even
/// the cheapest detour through 2 open areas is preferred over a single
/// closed-area hop (typical edge costs are ~20-50).
pub const CLOSED_PENALTY: u32 = 1000;

/// Snapshot of the area graph from one tribute's perspective at one
/// moment in time. Built per planning call — do not cache across cycles.
pub struct AreaGraph<'a> {
    /// All known areas (whether represented in `area_details` or not).
    /// Used to define the node set; missing details still produce a node
    /// connected by topology, just with default-terrain cost.
    pub areas: Vec<Area>,
    /// Tribute doing the planning. Used for stamina-cost computation.
    pub tribute: &'a Tribute,
    /// Per-area details lookup (terrain, items, etc.).
    pub details: HashMap<Area, &'a AreaDetails>,
    /// Set of areas currently closed (e.g., due to area events).
    pub closed: std::collections::HashSet<Area>,
}

impl<'a> AreaGraph<'a> {
    pub fn new(areas: &'a [AreaDetails], closed: &[Area], tribute: &'a Tribute) -> Self {
        let mut details: HashMap<Area, &AreaDetails> = HashMap::new();
        for ad in areas {
            if let Some(a) = ad.area {
                details.insert(a, ad);
            }
        }
        Self {
            areas: Area::iter().collect(),
            tribute,
            details,
            closed: closed.iter().copied().collect(),
        }
    }

    fn edge_cost(&self, to: Area) -> u32 {
        let detail = self.details.get(&to).copied();
        let stamina = if let Some(ad) = detail {
            calculate_stamina_cost(&Action::Move(Some(to)), &ad.terrain, self.tribute)
        } else {
            // No detail known — fall back to base move cost.
            20
        };
        let harshness = if let Some(ad) = detail {
            match ad.terrain.base.harshness() {
                Harshness::Mild => 0,
                Harshness::Moderate => 10,
                Harshness::Harsh => 20,
            }
        } else {
            0
        };
        let closed = if self.closed.contains(&to) {
            CLOSED_PENALTY
        } else {
            0
        };
        stamina + harshness + closed
    }
}

impl<'a> Graph for AreaGraph<'a> {
    type Node = Area;
    type Cost = u32;

    fn neighbors(&self, node: Area) -> Vec<(Area, u32)> {
        node.neighbors()
            .into_iter()
            .map(|n| (n, self.edge_cost(n)))
            .collect()
    }

    fn heuristic(&self, _from: Area, _to: Area) -> u32 {
        // Hop-count is 0 or 1 in this 7-node graph (Cornucopia is
        // adjacent to everything; sectors are at most 2 hops apart).
        // A constant zero (Dijkstra) is admissible and optimal here.
        0
    }
}

/// Plan a stamina-aware path from `start` to `goal`. Returns the full
/// path including endpoints and the total cost. Returns `None` only if
/// `goal` is unreachable from `start` (impossible in the v1 topology
/// since the graph is fully connected via Cornucopia).
pub fn plan_path(
    areas: &[AreaDetails],
    closed: &[Area],
    tribute: &Tribute,
    start: Area,
    goal: Area,
) -> Option<(Vec<Area>, u32)> {
    let g = AreaGraph::new(areas, closed, tribute);
    astar(&g, start, goal)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::{BaseTerrain, TerrainType};

    fn area(name: &str, a: Area, base: BaseTerrain) -> AreaDetails {
        AreaDetails::new_with_terrain(
            Some(name.to_string()),
            a,
            TerrainType::new(base, vec![]).unwrap(),
        )
    }

    fn standard_areas() -> Vec<AreaDetails> {
        vec![
            area("c", Area::Cornucopia, BaseTerrain::Clearing),
            area("s1", Area::Sector1, BaseTerrain::Forest),
            area("s2", Area::Sector2, BaseTerrain::Mountains),
            area("s3", Area::Sector3, BaseTerrain::Grasslands),
            area("s4", Area::Sector4, BaseTerrain::Desert),
            area("s5", Area::Sector5, BaseTerrain::Wetlands),
            area("s6", Area::Sector6, BaseTerrain::Tundra),
        ]
    }

    fn fresh_tribute() -> Tribute {
        Tribute::new("Pathfinder".to_string(), Some(0), None)
    }

    #[test]
    fn plan_to_self_returns_singleton() {
        let areas = standard_areas();
        let t = fresh_tribute();
        let (path, cost) = plan_path(&areas, &[], &t, Area::Cornucopia, Area::Cornucopia).unwrap();
        assert_eq!(path, vec![Area::Cornucopia]);
        assert_eq!(cost, 0);
    }

    #[test]
    fn plan_neighbor_is_two_node_path() {
        let areas = standard_areas();
        let t = fresh_tribute();
        let (path, _) = plan_path(&areas, &[], &t, Area::Cornucopia, Area::Sector1).unwrap();
        assert_eq!(path, vec![Area::Cornucopia, Area::Sector1]);
    }

    #[test]
    fn plan_opposite_sectors_routes_through_cornucopia() {
        // Sector1 (top-right) and Sector4 (bottom-left) are not adjacent.
        // The fastest route is Sector1 -> Cornucopia -> Sector4.
        let areas = standard_areas();
        let t = fresh_tribute();
        let (path, _) = plan_path(&areas, &[], &t, Area::Sector1, Area::Sector4).unwrap();
        assert_eq!(path.len(), 3);
        assert_eq!(path[0], Area::Sector1);
        assert_eq!(path[1], Area::Cornucopia);
        assert_eq!(path[2], Area::Sector4);
    }

    #[test]
    fn plan_routes_around_closed_area_when_possible() {
        // Sector1 -> Sector4 normally goes via Cornucopia. If Cornucopia
        // is closed, the routing must wrap around the ring (e.g. through
        // Sector2 + Sector3).
        let areas = standard_areas();
        let t = fresh_tribute();
        let closed = [Area::Cornucopia];
        let (path, cost) = plan_path(&areas, &closed, &t, Area::Sector1, Area::Sector4).unwrap();
        assert!(
            !path.contains(&Area::Cornucopia),
            "path detoured around closed cornucopia: {path:?}"
        );
        assert!(
            cost < CLOSED_PENALTY,
            "should not pay closed-penalty when an open route exists"
        );
    }

    #[test]
    fn plan_traverses_closed_area_as_last_resort() {
        // Close every ring sector, leaving only the path Cornucopia -><br/>
        // closed sector. Then Cornucopia -> Sector1 must traverse a
        // closed area and pay the penalty.
        let areas = standard_areas();
        let t = fresh_tribute();
        let closed = [Area::Sector1];
        let (path, cost) = plan_path(&areas, &closed, &t, Area::Cornucopia, Area::Sector1).unwrap();
        assert_eq!(path, vec![Area::Cornucopia, Area::Sector1]);
        assert!(
            cost >= CLOSED_PENALTY,
            "expected closed-penalty in cost, got {cost}"
        );
    }
}
