use crate::utils::union_find::UnionFind;
use crate::utils::counters::Counter;
use super::gate_fragments::GateFragment;

static GLOBAL_CIRCUIT_FRAGMENT_ID: Counter = Counter::new();

use std::collections::HashMap;

pub struct CircuitFragment {
    pub id: usize,
    pub gate_fragments: Vec<GateFragment>,
    pub qubit_edges: HashMap<usize, usize>,
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
            qubit_edges: HashMap::new(),
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
}