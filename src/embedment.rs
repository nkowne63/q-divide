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
        for (from_measurement_gate_fragments, target_gate_fragments_and_labels) in self.feedback_edges.edges.iter() {
            for from_measurement_gate_fragment in from_measurement_gate_fragments {
                for (target_gate_fragment, _) in target_gate_fragments_and_labels {
                    let target_unitary_id = self.control_edges.find(*target_gate_fragment);
                    edges.insert((*from_measurement_gate_fragment, target_unitary_id));
                }
            }
        }
        GateDependencyGraph { edges }
    }
    fn check_all_uniatry(&mut self) -> Result<(), String> {
        for (unitary_id, unitary_belongings) in self.control_edges.get_groups() {
            for fragment in unitary_belongings {
                match self.gate_fragments.get(fragment) {
                    None => return Err(format!("Gate Fragment {} not found at Circuit Fragment {}", fragment, self.id)),
                    Some(gate_fragment) if gate_fragment.is_measurement() => return Err(format!("Gate Fragment {} is a measurement at Circuit Fragment {}", fragment, self.id)),
                    Some(_) => {}
                }
            }
        }
        Ok(())
    }
}

impl GateDependencyGraph {
    // TODO: return not only bool but sorted indices
    fn has_loop(&self) -> bool {
        use petgraph::{
            algo::{toposort, DfsSpace},
            graph::DiGraph
        };
        // from usize to u32
        let g: DiGraph<(), ()> = DiGraph::from_edges(self.edges.iter().map(|(a, b)| (*a as u32, *b as u32)));
        let mut dfs = DfsSpace::new(&g);
        let result = toposort(&g, Some(&mut dfs));
        return result.is_err();
    }
}

impl TryInto<QuantumCircuit> for CircuitFragment {
    type Error = String;

    fn try_into(self) -> Result<QuantumCircuit, Self::Error> {
        todo!() // convert into quantum circuit
    }
}