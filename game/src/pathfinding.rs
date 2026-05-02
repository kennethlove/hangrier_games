//! Generic graph pathfinding (A*).
//!
//! Designed to be reused at multiple granularities — v1 operates on the
//! 7-area top-level hex graph; sub-tile pathfinding (hangrier_games-le8l)
//! will plug in a separate `Graph` impl over the sub-tile grid.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::hash::Hash;

/// A weighted, directed graph over `Node`s with associated cost type
/// `Cost`. Implementors expose neighbors and edge costs; pathfinding
/// operates entirely through this trait.
pub trait Graph {
    type Node: Copy + Eq + Hash;
    type Cost: Copy + Ord + Default + std::ops::Add<Output = Self::Cost>;

    /// Outgoing edges from `node`: `(neighbor, edge_cost)`.
    fn neighbors(&self, node: Self::Node) -> Vec<(Self::Node, Self::Cost)>;

    /// Admissible heuristic: lower bound on cost from `from` to `to`.
    /// Returning `Cost::default()` (zero) degrades A* to Dijkstra.
    fn heuristic(&self, from: Self::Node, to: Self::Node) -> Self::Cost;
}

/// Priority-queue entry. `BinaryHeap` is a max-heap, so we invert the
/// comparison on `f` (g + h) to get min-heap behavior.
struct Frontier<N, C> {
    f: C,
    node: N,
}

impl<N, C: Ord> Ord for Frontier<N, C> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f.cmp(&self.f)
    }
}
impl<N, C: Ord> PartialOrd for Frontier<N, C> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<N, C: Eq> Eq for Frontier<N, C> {}
impl<N, C: Eq> PartialEq for Frontier<N, C> {
    fn eq(&self, other: &Self) -> bool {
        self.f == other.f
    }
}

/// A* shortest path. Returns the path from `start` to `goal` inclusive,
/// and the total cost. Returns `None` if `goal` is unreachable.
pub fn astar<G: Graph>(
    graph: &G,
    start: G::Node,
    goal: G::Node,
) -> Option<(Vec<G::Node>, G::Cost)> {
    if start == goal {
        return Some((vec![start], G::Cost::default()));
    }

    let mut g_score: HashMap<G::Node, G::Cost> = HashMap::new();
    let mut came_from: HashMap<G::Node, G::Node> = HashMap::new();
    let mut open: BinaryHeap<Frontier<G::Node, G::Cost>> = BinaryHeap::new();

    g_score.insert(start, G::Cost::default());
    open.push(Frontier {
        f: graph.heuristic(start, goal),
        node: start,
    });

    while let Some(Frontier { node: current, .. }) = open.pop() {
        if current == goal {
            return Some(reconstruct(&came_from, current, g_score[&current]));
        }
        let current_g = g_score[&current];
        for (neighbor, edge_cost) in graph.neighbors(current) {
            let tentative_g = current_g + edge_cost;
            let better = g_score
                .get(&neighbor)
                .map(|&existing| tentative_g < existing)
                .unwrap_or(true);
            if better {
                came_from.insert(neighbor, current);
                g_score.insert(neighbor, tentative_g);
                let f = tentative_g + graph.heuristic(neighbor, goal);
                open.push(Frontier { f, node: neighbor });
            }
        }
    }

    None
}

fn reconstruct<N: Copy + Eq + Hash, C: Copy>(
    came_from: &HashMap<N, N>,
    end: N,
    cost: C,
) -> (Vec<N>, C) {
    let mut path = vec![end];
    let mut cur = end;
    while let Some(&prev) = came_from.get(&cur) {
        path.push(prev);
        cur = prev;
    }
    path.reverse();
    (path, cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Trivial test graph: nodes are u32, neighbors come from a HashMap.
    struct TestGraph {
        edges: HashMap<u32, Vec<(u32, u32)>>,
    }

    impl Graph for TestGraph {
        type Node = u32;
        type Cost = u32;
        fn neighbors(&self, node: u32) -> Vec<(u32, u32)> {
            self.edges.get(&node).cloned().unwrap_or_default()
        }
        fn heuristic(&self, _from: u32, _to: u32) -> u32 {
            0
        }
    }

    fn graph(pairs: &[(u32, u32, u32)]) -> TestGraph {
        let mut edges: HashMap<u32, Vec<(u32, u32)>> = HashMap::new();
        for &(a, b, c) in pairs {
            edges.entry(a).or_default().push((b, c));
            edges.entry(b).or_default().push((a, c));
        }
        TestGraph { edges }
    }

    #[test]
    fn start_equals_goal_returns_singleton() {
        let g = graph(&[]);
        let (path, cost) = astar(&g, 5, 5).unwrap();
        assert_eq!(path, vec![5]);
        assert_eq!(cost, 0);
    }

    #[test]
    fn unreachable_returns_none() {
        let g = graph(&[(1, 2, 1)]);
        assert!(astar(&g, 1, 99).is_none());
    }

    #[test]
    fn picks_lowest_cost_path_not_fewest_hops() {
        // Direct edge 1->3 with cost 100; detour 1->2->3 with cost 1+1=2.
        let g = graph(&[(1, 3, 100), (1, 2, 1), (2, 3, 1)]);
        let (path, cost) = astar(&g, 1, 3).unwrap();
        assert_eq!(path, vec![1, 2, 3]);
        assert_eq!(cost, 2);
    }

    #[test]
    fn finds_three_hop_path() {
        let g = graph(&[(1, 2, 1), (2, 3, 1), (3, 4, 1)]);
        let (path, cost) = astar(&g, 1, 4).unwrap();
        assert_eq!(path, vec![1, 2, 3, 4]);
        assert_eq!(cost, 3);
    }

    #[test]
    fn ties_resolved_consistently() {
        let g = graph(&[(1, 2, 1), (1, 3, 1), (2, 4, 1), (3, 4, 1)]);
        let (path, cost) = astar(&g, 1, 4).unwrap();
        assert_eq!(cost, 2);
        assert_eq!(path.len(), 3);
        assert_eq!(path[0], 1);
        assert_eq!(path[2], 4);
    }
}
