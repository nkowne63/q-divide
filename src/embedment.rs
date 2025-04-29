use std::collections::HashSet;

use super::circuits::QuantumCircuit;
use super::fragments::circuit_fragments::CircuitFragment;

struct GateDependencyGraph {
    edges: HashSet<(usize, usize)>
}

impl CircuitFragment {
    fn get_next(&self, idx: usize) -> Option<usize> {
        self.qubit_edges.edges.get(&idx).copied()
    }
    fn into_dependency_graph(&mut self) -> GateDependencyGraph {
        let mut edges = HashSet::new();
        // build unitary connectivity
        for (unitary_id, unitary_belongings) in self.control_edges.get_groups() {
            for fragment_id in unitary_belongings {
                if let Some(next) = self.get_next(fragment_id) {
                    let next_unitary_id = self.control_edges.find(next);
                    edges.insert((unitary_id, next_unitary_id));
                }
            }
        }
        // build measurement connectivity
        for label in self.feedback_edges.labels.keys() {
            todo!(); // insert deps for measurements
        }
        GateDependencyGraph { edges }
    }
    fn check_all_uniatry(self) -> Result<(), String> {
        todo!() // check all control_edges groups are valid unitary
    }
}

impl GateDependencyGraph {
    fn has_loop(&self) -> bool {
        todo!() // check and translate into QC
    }
}

impl TryInto<QuantumCircuit> for CircuitFragment {
    type Error = String;

    fn try_into(self) -> Result<QuantumCircuit, Self::Error> {
        todo!() // convert into quantum circuit
    }
}