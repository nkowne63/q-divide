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
        let get_gate_fragment_label = |id: &usize| {
            self.gate_fragments.iter().find(|gf| gf.id == *id).unwrap().label.clone()
        };
        for (unitary_id, unitary_belongings) in self.control_edges.get_groups() {
            // ensure no measurement in unitaries
            let no_measurement = unitary_belongings.iter().filter(|idx| get_gate_fragment_label(*idx).is_measurement()).count() == 0;
            // ensure at least one unitary
            let has_unitary = unitary_belongings.iter().filter(|idx| get_gate_fragment_label(*idx).is_unitary()).count() > 0;
            if !no_measurement || !has_unitary {
                return Err(format!("The group contains GateFragment {} has no unitary or measurement", unitary_id));
            }
        }
        Ok(())
    }
}

impl GateDependencyGraph {
    fn toposort(&self) -> Result<Vec<usize>, String> {
        use petgraph::{
            algo::{toposort, DfsSpace},
            graph::DiGraph
        };
        let edges = self.edges.iter().map(|(a, b)| (*a, *b)).collect::<Vec<_>>();
        let mut occuring_indices = edges.iter().flat_map(|(a, b)| vec![*a, *b]).collect::<HashSet<_>>().into_iter().collect::<Vec<_>>();
        occuring_indices.sort();
        let occuring_indices_sorted = occuring_indices.clone();
        let edges_indices_mapped = edges.iter().map(|(a, b)| {
            // map edge to ocuuring_indices index
            let a_mapped = occuring_indices.iter().position(|x| x == a).unwrap() as u32;
            let b_mapped = occuring_indices.iter().position(|x| x == b).unwrap() as u32;
            return (a_mapped, b_mapped)
        }).collect::<Vec<_>>();
        // from usize to u32
        let g: DiGraph<(), ()> = DiGraph::from_edges(edges_indices_mapped.iter().map(|(a, b)| (*a as u32, *b as u32)));
        let mut dfs = DfsSpace::new(&g);
        let result = toposort(&g, Some(&mut dfs)).map_err(|err| {
            format!("Graph has a loop including {:?}", err.node_id().index())
        });
        let inverse_mapped = result.map(|result| {
            result.iter().map(|x| *occuring_indices_sorted.get(x.index()).unwrap()).collect::<Vec<_>>()
        });
        return inverse_mapped;
    }
}

impl TryInto<QuantumCircuit> for CircuitFragment {
    type Error = String;

    fn try_into(mut self) -> Result<QuantumCircuit, Self::Error> {
        self.check_all_uniatry()?;
        let toposorted = self.into_dependency_graph().toposort()?;
        let _fragment_groups = toposorted.into_iter().map(|gate_fragment_id| {
            let _fragments = self.control_edges.get_same_group(gate_fragment_id).iter().map(|id| {
                self.gate_fragments.iter().find(|gf| gf.id == *id).unwrap().clone()
            }).collect::<Vec<_>>();
            todo!() // add measurement dependency + convert to quantum gate
        }).collect::<Vec<_>>();

        todo!() // convert into quantum circuit
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dependency_graph_has_loop() {
        let g = GateDependencyGraph {
            edges: vec![(0, 1), (1, 2), (2, 0)].into_iter().collect()
        };
        assert_eq!(g.toposort(), Err("Graph has a loop including 2".to_string()));
    }
    #[test]
    fn dependency_graph_no_loop() {
        let g = GateDependencyGraph {
            edges: vec![(0, 1), (1, 2)].into_iter().collect()
        };
        assert!(g.toposort().is_ok());
    }
    #[test]
    fn dependency_graph_empty() {
        let g = GateDependencyGraph {
            edges: HashSet::new()
        };
        assert!(g.toposort().is_ok());
    }
    #[test]
    fn dependency_graph_leap() {
        let g = GateDependencyGraph {
            edges: vec![(0, 5), (5, 2), (2, 3)].into_iter().collect()
        };
        assert!(g.toposort().is_ok());
        assert_eq!(g.toposort().unwrap(), vec![0, 5, 2, 3]);
    }
}