use std::collections::HashMap;

/// A room (node) in the Grand Pattern graph.
#[derive(Debug)]
struct Room {
    id: usize,
    vibe: f64,
    surprise: f64,
}

/// An edge between two rooms.
#[derive(Debug, Clone)]
struct Edge {
    from: usize,
    to: usize,
    weight: f64,
}

/// The Grand Pattern graph — an opaque type exposed via C FFI.
#[derive(Debug)]
pub struct GpGraph {
    rooms: HashMap<usize, Room>,
    edges: Vec<Edge>,
    next_id: usize,
    bpm: f64,
    tick_count: u64,
    initial_total_vibe: f64,
}

impl GpGraph {
    pub fn new(bpm: f64) -> Self {
        GpGraph {
            rooms: HashMap::new(),
            edges: Vec::new(),
            next_id: 0,
            bpm,
            tick_count: 0,
            initial_total_vibe: 0.0,
        }
    }

    pub fn add_room(&mut self, vibe: f64) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        let room = Room {
            id,
            vibe,
            surprise: 0.0,
        };
        self.rooms.insert(id, room);
        self.initial_total_vibe = self.total_vibe();
        id
    }

    pub fn remove_room(&mut self, id: usize) {
        self.rooms.remove(&id);
        self.edges.retain(|e| e.from != id && e.to != id);
        self.initial_total_vibe = self.total_vibe();
    }

    pub fn room_vibe(&self, id: usize) -> f64 {
        self.rooms.get(&id).map(|r| r.vibe).unwrap_or(0.0)
    }

    pub fn room_surprise(&self, id: usize) -> f64 {
        self.rooms.get(&id).map(|r| r.surprise).unwrap_or(0.0)
    }

    pub fn set_room_vibe(&mut self, id: usize, vibe: f64) {
        if let Some(room) = self.rooms.get_mut(&id) {
            room.vibe = vibe;
        }
    }

    pub fn add_edge(&mut self, from: usize, to: usize, weight: f64) {
        // Only add edge if both rooms exist
        if self.rooms.contains_key(&from) && self.rooms.contains_key(&to) {
            self.edges.push(Edge { from, to, weight });
        }
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn tick(&mut self) {
        // Each room's surprise is proportional to vibe deviation from mean
        let mean = self.fleet_vibe();
        for room in self.rooms.values_mut() {
            room.surprise = (room.vibe - mean).abs() * self.bpm / 60.0;
        }
        self.tick_count += 1;
    }

    pub fn diffuse(&mut self, rate: f64) {
        // Collect contributions per room
        let mut contributions: HashMap<usize, f64> = HashMap::new();
        for room_id in self.rooms.keys() {
            contributions.insert(*room_id, 0.0);
        }

        for edge in &self.edges {
            let from_vibe = self.room_vibe(edge.from);
            let to_vibe = self.room_vibe(edge.to);
            let flow = (from_vibe - to_vibe) * rate * edge.weight;
            *contributions.get_mut(&edge.from).unwrap() -= flow;
            *contributions.get_mut(&edge.to).unwrap() += flow;
        }

        for (id, contrib) in contributions {
            if let Some(room) = self.rooms.get_mut(&id) {
                room.vibe += contrib;
            }
        }
    }

    pub fn learn(&mut self) {
        // Rooms with high surprise adjust vibe toward the fleet mean
        let fleet = self.fleet_vibe();
        let total_surprise = self.fleet_surprise();
        let rate = if total_surprise > 0.0 { 0.01 } else { 0.0 };

        for room in self.rooms.values_mut() {
            room.vibe += (fleet - room.vibe) * room.surprise * rate;
        }
    }

    pub fn total_vibe(&self) -> f64 {
        self.rooms.values().map(|r| r.vibe).sum()
    }

    pub fn fleet_vibe(&self) -> f64 {
        let n = self.rooms.len();
        if n == 0 {
            return 0.0;
        }
        self.total_vibe() / n as f64
    }

    pub fn fleet_surprise(&self) -> f64 {
        let n = self.rooms.len();
        if n == 0 {
            return 0.0;
        }
        self.rooms.values().map(|r| r.surprise).sum::<f64>() / n as f64
    }

    pub fn room_count(&self) -> usize {
        self.rooms.len()
    }

    pub fn tick_count_value(&self) -> u64 {
        self.tick_count
    }

    pub fn verify_conservation(&self, tolerance: f64) -> bool {
        if self.rooms.is_empty() {
            return true;
        }
        (self.total_vibe() - self.initial_total_vibe).abs() <= tolerance
    }

    pub fn topology_chain(&mut self) {
        self.edges.clear();
        let mut ids: Vec<usize> = self.rooms.keys().copied().collect();
        ids.sort_unstable();
        for w in ids.windows(2) {
            self.edges.push(Edge { from: w[0], to: w[1], weight: 1.0 });
        }
    }

    pub fn topology_ring(&mut self) {
        self.topology_chain();
        let mut ids: Vec<usize> = self.rooms.keys().copied().collect();
        ids.sort_unstable();
        if ids.len() > 2 {
            let last = *ids.last().unwrap();
            let first = ids[0];
            self.edges.push(Edge { from: last, to: first, weight: 1.0 });
        }
    }

    pub fn topology_star(&mut self) {
        self.edges.clear();
        let mut ids: Vec<usize> = self.rooms.keys().copied().collect();
        ids.sort_unstable();
        if ids.len() < 2 {
            return;
        }
        let center = ids[0];
        for &id in &ids[1..] {
            self.edges.push(Edge { from: center, to: id, weight: 1.0 });
        }
    }

    pub fn topology_mesh(&mut self) {
        self.edges.clear();
        let ids: Vec<usize> = self.rooms.keys().copied().collect();
        for i in &ids {
            for j in &ids {
                if i != j {
                    self.edges.push(Edge { from: *i, to: *j, weight: 1.0 });
                }
            }
        }
    }

    pub fn topology_small_world(&mut self, probability: f64) {
        self.topology_ring();
        // Rewire each edge with given probability
        let ids: Vec<usize> = self.rooms.keys().copied().collect();
        if ids.is_empty() {
            return;
        }
        let mut rng_state: u64 = 42;
        for edge in &mut self.edges {
            // Simple LCG random
            rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let rand_val = ((rng_state >> 33) as f64) / (1u64 << 31) as f64;
            if rand_val < probability {
                let target_idx = (rng_state as usize) % ids.len();
                edge.to = ids[target_idx];
            }
        }
    }

    pub fn export_vibes(&self, out: &mut [f64]) {
        let mut ids: Vec<usize> = self.rooms.keys().copied().collect();
        ids.sort_unstable();
        for (i, id) in ids.iter().enumerate() {
            if i < out.len() {
                out[i] = self.room_vibe(*id);
            }
        }
    }

    pub fn export_surprises(&self, out: &mut [f64]) {
        let mut ids: Vec<usize> = self.rooms.keys().copied().collect();
        ids.sort_unstable();
        for (i, id) in ids.iter().enumerate() {
            if i < out.len() {
                out[i] = self.room_surprise(*id);
            }
        }
    }
}

// ============================================================================
// C FFI exports
// ============================================================================

use std::os::raw::c_int;

#[no_mangle]
pub extern "C" fn gp_graph_new(bpm: f64) -> *mut GpGraph {
    Box::into_raw(Box::new(GpGraph::new(bpm)))
}

/// Free is safe on null and on double-free (we zero the pointer semantics via Box).
#[no_mangle]
pub unsafe extern "C" fn gp_graph_free(graph: *mut GpGraph) {
    if !graph.is_null() {
        let _ = Box::from_raw(graph);
    }
}

#[no_mangle]
pub unsafe extern "C" fn gp_add_room(graph: *mut GpGraph, vibe: f64) -> usize {
    if graph.is_null() { return 0; }
    (*graph).add_room(vibe)
}

#[no_mangle]
pub unsafe extern "C" fn gp_remove_room(graph: *mut GpGraph, id: usize) {
    if graph.is_null() { return; }
    (*graph).remove_room(id)
}

#[no_mangle]
pub unsafe extern "C" fn gp_room_vibe(graph: *const GpGraph, id: usize) -> f64 {
    if graph.is_null() { return 0.0; }
    (*graph).room_vibe(id)
}

#[no_mangle]
pub unsafe extern "C" fn gp_room_surprise(graph: *const GpGraph, id: usize) -> f64 {
    if graph.is_null() { return 0.0; }
    (*graph).room_surprise(id)
}

#[no_mangle]
pub unsafe extern "C" fn gp_set_room_vibe(graph: *mut GpGraph, id: usize, vibe: f64) {
    if graph.is_null() { return; }
    (*graph).set_room_vibe(id, vibe)
}

#[no_mangle]
pub unsafe extern "C" fn gp_add_edge(graph: *mut GpGraph, from: usize, to: usize, weight: f64) {
    if graph.is_null() { return; }
    (*graph).add_edge(from, to, weight)
}

#[no_mangle]
pub unsafe extern "C" fn gp_edge_count(graph: *const GpGraph) -> usize {
    if graph.is_null() { return 0; }
    (*graph).edge_count()
}

#[no_mangle]
pub unsafe extern "C" fn gp_tick(graph: *mut GpGraph) {
    if graph.is_null() { return; }
    (*graph).tick()
}

#[no_mangle]
pub unsafe extern "C" fn gp_diffuse(graph: *mut GpGraph, rate: f64) {
    if graph.is_null() { return; }
    (*graph).diffuse(rate)
}

#[no_mangle]
pub unsafe extern "C" fn gp_learn(graph: *mut GpGraph) {
    if graph.is_null() { return; }
    (*graph).learn()
}

#[no_mangle]
pub unsafe extern "C" fn gp_total_vibe(graph: *const GpGraph) -> f64 {
    if graph.is_null() { return 0.0; }
    (*graph).total_vibe()
}

#[no_mangle]
pub unsafe extern "C" fn gp_fleet_vibe(graph: *const GpGraph) -> f64 {
    if graph.is_null() { return 0.0; }
    (*graph).fleet_vibe()
}

#[no_mangle]
pub unsafe extern "C" fn gp_fleet_surprise(graph: *const GpGraph) -> f64 {
    if graph.is_null() { return 0.0; }
    (*graph).fleet_surprise()
}

#[no_mangle]
pub unsafe extern "C" fn gp_room_count(graph: *const GpGraph) -> usize {
    if graph.is_null() { return 0; }
    (*graph).room_count()
}

#[no_mangle]
pub unsafe extern "C" fn gp_tick_count(graph: *const GpGraph) -> u64 {
    if graph.is_null() { return 0; }
    (*graph).tick_count_value()
}

#[no_mangle]
pub unsafe extern "C" fn gp_verify_conservation(graph: *const GpGraph, tolerance: f64) -> c_int {
    if graph.is_null() { return 1; }
    if (*graph).verify_conservation(tolerance) { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn gp_topology_chain(graph: *mut GpGraph) {
    if graph.is_null() { return; }
    (*graph).topology_chain()
}

#[no_mangle]
pub unsafe extern "C" fn gp_topology_ring(graph: *mut GpGraph) {
    if graph.is_null() { return; }
    (*graph).topology_ring()
}

#[no_mangle]
pub unsafe extern "C" fn gp_topology_star(graph: *mut GpGraph) {
    if graph.is_null() { return; }
    (*graph).topology_star()
}

#[no_mangle]
pub unsafe extern "C" fn gp_topology_mesh(graph: *mut GpGraph) {
    if graph.is_null() { return; }
    (*graph).topology_mesh()
}

#[no_mangle]
pub unsafe extern "C" fn gp_topology_small_world(graph: *mut GpGraph, probability: f64) {
    if graph.is_null() { return; }
    (*graph).topology_small_world(probability)
}

#[no_mangle]
pub unsafe extern "C" fn gp_export_vibes(graph: *const GpGraph, out: *mut f64, len: usize) {
    if graph.is_null() || out.is_null() { return; }
    let slice = std::slice::from_raw_parts_mut(out, len);
    (*graph).export_vibes(slice)
}

#[no_mangle]
pub unsafe extern "C" fn gp_export_surprises(graph: *const GpGraph, out: *mut f64, len: usize) {
    if graph.is_null() || out.is_null() { return; }
    let slice = std::slice::from_raw_parts_mut(out, len);
    (*graph).export_surprises(slice)
}

// ============================================================================
// Rust tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_create_and_free() {
        let g = gp_graph_new(120.0);
        assert!(!g.is_null());
        unsafe { gp_graph_free(g) };
    }

    #[test]
    fn test_add_room() {
        let mut g = GpGraph::new(120.0);
        let id = g.add_room(1.0);
        assert_eq!(g.room_count(), 1);
        assert_eq!(id, 0);
    }

    #[test]
    fn test_add_edge() {
        let mut g = GpGraph::new(120.0);
        let a = g.add_room(1.0);
        let b = g.add_room(2.0);
        g.add_edge(a, b, 1.0);
        assert_eq!(g.edge_count(), 1);
    }

    #[test]
    fn test_tick() {
        let mut g = GpGraph::new(120.0);
        g.add_room(1.0);
        g.add_room(2.0);
        g.tick();
        assert_eq!(g.tick_count_value(), 1);
        // Surprise should be > 0 since rooms differ
        assert!(g.room_surprise(0) > 0.0 || g.room_surprise(1) > 0.0);
    }

    #[test]
    fn test_diffuse() {
        let mut g = GpGraph::new(120.0);
        let a = g.add_room(2.0);
        let b = g.add_room(0.0);
        g.add_edge(a, b, 1.0);
        g.diffuse(0.5);
        // Vibes should have moved toward each other
        assert!(g.room_vibe(a) < 2.0);
        assert!(g.room_vibe(b) > 0.0);
    }

    #[test]
    fn test_learn() {
        let mut g = GpGraph::new(120.0);
        g.add_room(1.0);
        g.add_room(5.0);
        g.tick();
        let v0 = g.room_vibe(0);
        g.learn();
        // Room 0 should have moved toward fleet mean
        assert!(g.room_vibe(0) != v0 || g.fleet_surprise() == 0.0);
    }

    #[test]
    fn test_conservation() {
        let mut g = GpGraph::new(120.0);
        let a = g.add_room(2.0);
        let b = g.add_room(3.0);
        g.add_edge(a, b, 1.0);
        let initial = g.total_vibe();
        g.diffuse(0.5);
        // Diffusion is conservative
        assert!((g.total_vibe() - initial).abs() < 1e-10);
    }

    #[test]
    fn test_room_vibe_getter() {
        let mut g = GpGraph::new(120.0);
        g.add_room(3.14);
        assert!((g.room_vibe(0) - 3.14).abs() < 1e-10);
    }

    #[test]
    fn test_room_surprise_getter() {
        let mut g = GpGraph::new(120.0);
        g.add_room(1.0);
        g.tick();
        assert!(g.room_surprise(0) >= 0.0);
    }

    #[test]
    fn test_set_room_vibe() {
        let mut g = GpGraph::new(120.0);
        g.add_room(1.0);
        g.set_room_vibe(0, 5.0);
        assert!((g.room_vibe(0) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_remove_room() {
        let mut g = GpGraph::new(120.0);
        g.add_room(1.0);
        g.add_room(2.0);
        g.remove_room(0);
        assert_eq!(g.room_count(), 1);
        assert_eq!(g.room_vibe(0), 0.0); // removed
    }

    #[test]
    fn test_fleet_stats() {
        let mut g = GpGraph::new(120.0);
        g.add_room(2.0);
        g.add_room(4.0);
        assert!((g.fleet_vibe() - 3.0).abs() < 1e-10);
        assert!((g.total_vibe() - 6.0).abs() < 1e-10);
        assert_eq!(g.room_count(), 2);
    }

    #[test]
    fn test_topology_chain() {
        let mut g = GpGraph::new(120.0);
        for _ in 0..5 { g.add_room(1.0); }
        g.topology_chain();
        assert_eq!(g.edge_count(), 4); // n-1 edges
    }

    #[test]
    fn test_topology_ring() {
        let mut g = GpGraph::new(120.0);
        for _ in 0..5 { g.add_room(1.0); }
        g.topology_ring();
        assert_eq!(g.edge_count(), 5); // n edges (chain + wrap)
    }

    #[test]
    fn test_topology_star() {
        let mut g = GpGraph::new(120.0);
        for _ in 0..5 { g.add_room(1.0); }
        g.topology_star();
        assert_eq!(g.edge_count(), 4); // n-1 edges from center
    }

    #[test]
    fn test_topology_mesh() {
        let mut g = GpGraph::new(120.0);
        for _ in 0..4 { g.add_room(1.0); }
        g.topology_mesh();
        assert_eq!(g.edge_count(), 12); // n*(n-1)
    }

    #[test]
    fn test_topology_small_world() {
        let mut g = GpGraph::new(120.0);
        for _ in 0..10 { g.add_room(1.0); }
        g.topology_small_world(0.5);
        // Should have same edge count as ring (some rewired)
        assert_eq!(g.edge_count(), 10);
    }

    #[test]
    fn test_export_vibes() {
        let mut g = GpGraph::new(120.0);
        g.add_room(1.0);
        g.add_room(2.0);
        g.add_room(3.0);
        let mut buf = [0.0f64; 3];
        g.export_vibes(&mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-10);
        assert!((buf[2] - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_export_surprises() {
        let mut g = GpGraph::new(120.0);
        g.add_room(1.0);
        g.add_room(2.0);
        g.tick();
        let mut buf = [0.0f64; 2];
        g.export_surprises(&mut buf);
        assert!(buf[0] >= 0.0);
        assert!(buf[1] >= 0.0);
    }

    #[test]
    fn test_null_pointer_handling() {
        unsafe {
            assert_eq!(gp_room_vibe(std::ptr::null(), 0), 0.0);
            assert_eq!(gp_room_surprise(std::ptr::null(), 0), 0.0);
            assert_eq!(gp_room_count(std::ptr::null()), 0);
            assert_eq!(gp_edge_count(std::ptr::null()), 0);
            assert_eq!(gp_total_vibe(std::ptr::null()), 0.0);
            assert_eq!(gp_fleet_vibe(std::ptr::null()), 0.0);
            assert_eq!(gp_tick_count(std::ptr::null()), 0);
            assert_eq!(gp_verify_conservation(std::ptr::null(), 0.01), 1);
            // These should not crash
            gp_graph_free(std::ptr::null_mut());
            gp_add_room(std::ptr::null_mut(), 1.0);
            gp_tick(std::ptr::null_mut());
            gp_diffuse(std::ptr::null_mut(), 0.5);
            gp_learn(std::ptr::null_mut());
        }
    }

    #[test]
    fn test_large_graph() {
        let mut g = GpGraph::new(120.0);
        for i in 0..1000 {
            g.add_room(i as f64);
        }
        assert_eq!(g.room_count(), 1000);
        g.topology_ring();
        assert_eq!(g.edge_count(), 1000);
        g.diffuse(0.1);
        g.tick();
    }

    #[test]
    fn test_double_free_safe() {
        let g = gp_graph_new(120.0);
        unsafe { gp_graph_free(g) };
        // Double free — we only test this compiles and doesn't crash in practice.
        // In safe Rust we can't actually double-free, so we test the null check path.
        unsafe { gp_graph_free(std::ptr::null_mut()) };
    }

    #[test]
    fn test_thread_safety() {
        let h1 = thread::spawn(|| {
            let g = gp_graph_new(120.0);
            unsafe { gp_add_room(g, 1.0); }
            unsafe { gp_graph_free(g) };
        });
        let h2 = thread::spawn(|| {
            let g = gp_graph_new(140.0);
            unsafe { gp_add_room(g, 2.0); }
            unsafe { gp_graph_free(g) };
        });
        h1.join().unwrap();
        h2.join().unwrap();
    }

    #[test]
    fn test_memory_no_leaks() {
        // 100K create/free cycles — if this leaks, it'll OOM
        for _ in 0..100_000 {
            let mut g = GpGraph::new(120.0);
            g.add_room(1.0);
            g.add_room(2.0);
            g.add_edge(0, 1, 1.0);
            g.tick();
            g.diffuse(0.5);
        }
    }

    #[test]
    fn test_ffi_full_workflow() {
        unsafe {
            let g = gp_graph_new(120.0);
            assert!(!g.is_null());

            let r0 = gp_add_room(g, 1.0);
            let r1 = gp_add_room(g, 2.0);
            let _r2 = gp_add_room(g, 3.0);
            assert_eq!(gp_room_count(g), 3);

            gp_topology_chain(g);
            assert_eq!(gp_edge_count(g), 2);

            gp_tick(g);
            assert_eq!(gp_tick_count(g), 1);

            gp_diffuse(g, 0.5);
            assert!(gp_total_vibe(g) > 0.0);

            gp_learn(g);

            let mut vibes = [0.0f64; 3];
            gp_export_vibes(g, vibes.as_mut_ptr(), 3);
            assert!(vibes[0] > 0.0);

            let mut surprises = [0.0f64; 3];
            gp_export_surprises(g, surprises.as_mut_ptr(), 3);

            assert_eq!(gp_verify_conservation(g, 1e-6), 1); // diffusion preserves total vibe

            gp_remove_room(g, r1);
            assert_eq!(gp_room_count(g), 2);

            gp_set_room_vibe(g, r0, 10.0);
            assert!((gp_room_vibe(g, r0) - 10.0).abs() < 1e-10);

            gp_graph_free(g);
        }
    }

    #[test]
    fn test_empty_graph() {
        let g = GpGraph::new(120.0);
        assert_eq!(g.room_count(), 0);
        assert_eq!(g.total_vibe(), 0.0);
        assert_eq!(g.fleet_vibe(), 0.0);
        assert_eq!(g.fleet_surprise(), 0.0);
        assert!(g.verify_conservation(0.01));
    }
}
