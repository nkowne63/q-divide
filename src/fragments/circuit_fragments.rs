use crate::utils::counters::Counter;
use super::gate_fragments::GateFragment;

static GLOBAL_CIRCUIT_FRAGMENT_ID: Counter = Counter::new();

use petgraph::unionfind::UnionFind;
use std::collections::HashMap;

pub struct CircuitFragment {
    pub id: usize,
    pub gate_fragments: Vec<GateFragment>,
    pub qubit_edges: HashMap<usize, usize>,
    pub control_edges: UnionFind<usize>,
    pub feedback_edges: HashMap<usize, Vec<(usize, String)>>,
}

impl CircuitFragment {
    pub fn new(gate_fragments: Vec<GateFragment>) -> Self {
        let id = GLOBAL_CIRCUIT_FRAGMENT_ID.increment();
        let len = gate_fragments.len();
        let feedback_edges = {
            let mut feedback_edges = HashMap::new();
            for (index, fragment) in gate_fragments.iter().enumerate() {
                if fragment.label.is_measurement() {
                    feedback_edges.insert(index, Vec::new());
                }
            }
            feedback_edges
        };
        CircuitFragment {
            id,
            gate_fragments: gate_fragments,
            qubit_edges: HashMap::new(),
            control_edges: UnionFind::new(len),
            feedback_edges,
        }
    }
}