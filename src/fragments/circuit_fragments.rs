use crate::utils::union_find::UnionFind;
use crate::utils::counters::Counter;
use super::gate_fragments::GateFragment;

static GLOBAL_CIRCUIT_FRAGMENT_ID: Counter = Counter::new();

use std::collections::HashMap;

pub struct BacktrackableEdges {
    edges: HashMap<usize, usize>,
    backtrack: HashMap<usize, usize>,
}

impl BacktrackableEdges {
    fn new() -> Self {
        BacktrackableEdges {
            edges: HashMap::new(),
            backtrack: HashMap::new(),
        }
    }
    fn insert(&mut self, a: usize, b: usize) {
        self.edges.insert(a, b);
        self.backtrack.insert(b, a);
    }
    fn remove(&mut self, a: usize) {
        let b = self.edges.remove(&a).unwrap();
        self.backtrack.remove(&b);
    }
}

pub struct CircuitFragment {
    pub id: usize,
    pub gate_fragments: Vec<GateFragment>,
    pub qubit_edges: BacktrackableEdges,
    pub control_edges: UnionFind,
    pub feedback_edges: HashMap<Vec<usize>, Vec<(usize, String)>>,
}

impl CircuitFragment {
    pub fn new(gate_fragments: Vec<GateFragment>) -> Self {
        let id = GLOBAL_CIRCUIT_FRAGMENT_ID.increment();
        let len = gate_fragments.len();
        CircuitFragment {
            id,
            gate_fragments: gate_fragments,
            qubit_edges: BacktrackableEdges::new(),
            control_edges: UnionFind::new(len),
            feedback_edges: HashMap::new(),
        }
    }

    pub fn add_gate_fragment(&mut self, gate_fragment: GateFragment) -> Result<(), String> {
        self.gate_fragments.push(gate_fragment);
        self.control_edges.ensure(self.gate_fragments.len());

        Ok(())
    }

    pub fn remove_gate_fragment(&mut self, id: usize) -> Result<(), String> {
        self.gate_fragments.retain(|fragment| fragment.id != id as usize);
        self.control_edges.remove(id);
        Ok(())
    }

    pub fn connect_qubit_edges(&mut self, source: usize, destination: usize) -> Result<(), String> {
        if self.qubit_edges.edges.contains_key(&source) {
            return Err(format!("GateFragment {} is already connected as start", source));
        };
        if self.qubit_edges.backtrack.contains_key(&destination) {
            return Err(format!("GateFragment {} is already connected as end", destination));
        };
        self.qubit_edges.insert(source, destination);
        Ok(())
    }
    pub fn disconnect_qubit_edges(&mut self, source: usize) -> Result<(), String> {
        if !self.qubit_edges.edges.contains_key(&source) {
            return Err(format!("GateFragment {} is not connected as start", source));
        };
        self.qubit_edges.remove(source);
        Ok(())
    }
    pub fn unite_control_edges(&mut self, a: usize, b: usize) -> Result<(), String> {
        self.control_edges.union(a, b);
        Ok(())
    }
    pub fn eject_control_edges(&mut self, a: usize) -> Result<(), String> {
        self.control_edges.remove(a);
        Ok(())
    }
    pub fn connect_feedback_edges(&mut self, measurements: Vec<usize>, target: usize, label: String) -> Result<(), String> {
        todo!()
    }
    pub fn remove_feedback_edges(&mut self, label: String) -> Result<(), String> {
        todo!()
    }
}