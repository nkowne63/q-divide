use std::collections::HashSet;

use super::circuits::QuantumCircuit;
use super::fragments::circuit_fragments::CircuitFragment;

// WIP: quantum circuit fragments into quantum circuits and validation

struct GateDependencyGraph {
    edges: HashSet<(usize, usize)>
}

impl CircuitFragment {
    fn get_next(&self, idx: usize) -> Option<usize> {
        self.qubit_edges.edges.get(&idx).copied()
    }
    fn into_dependency_graph(&mut self) -> GateDependencyGraph {
        let mut edges = HashSet::new();
        // assert!(self.check_all_uniatry().is_ok(), "All control_edges groups must be valid unitary");
        // build unitary connectivity
        for (unitary_id, unitary_belongings) in self.control_edges.get_groups() {
            for fragment_id in unitary_belongings {
                if let Some(next) = self.get_next(fragment_id) {
                    // TODO: ensure find next is always unitary
                    let next_unitary_id = self.control_edges.find(next);
                    edges.insert((unitary_id, next_unitary_id));
                }
            }
        }
        // build measurement connectivity
        for label in self.feedback_edges.labels.keys() {
            // TODO: insert deps for measurements
        }
        GateDependencyGraph { edges }
    }
    // TODO: check all control_edges groups are valid unitary
    fn check_all_uniatry(self) -> Result<(), String> {
        todo!()
    }
}

impl GateDependencyGraph {
    // TODO: check and translate into QC
    fn has_loop(&self) -> bool {
        todo!()
    }
}

impl TryInto<QuantumCircuit> for CircuitFragment {
    type Error = String;

    fn try_into(self) -> Result<QuantumCircuit, Self::Error> {
        todo!()
    }
}