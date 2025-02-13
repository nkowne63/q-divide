use std::collections::HashMap;

use crate::utils::counters::Counter;
use super::gate_fragments::GateFragment;

static GLOBAL_CIRCUIT_FRAGMENT_ID: Counter = Counter::new();

pub struct CircuitFragment {
    pub id: usize,
    pub gate_fragments: Vec<GateFragment>,
    pub qubit_edges: HashMap<usize, usize>,
    // TODO: union-find data for control_edges
    // union-find + addable / deletable
}

impl CircuitFragment {
    pub fn new(gate_fragments: Vec<GateFragment>) -> Self {
        let id = GLOBAL_CIRCUIT_FRAGMENT_ID.increment();
        CircuitFragment {
            id,
            gate_fragments,
            qubit_edges: HashMap::new(),
        };
        todo!()
    }
}