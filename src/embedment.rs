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
        let get_gate_fragment_label = |idx: &usize| {
            if *idx < self.gate_fragments.len() {
                Some(self.gate_fragments[*idx].label.clone())
            } else {
                None
            }
        };
        for (unitary_id, unitary_belongings) in self.control_edges.get_groups() {
            let measurement_count = unitary_belongings.iter()
                .filter_map(|idx| get_gate_fragment_label(idx))
                .filter(|label| label.is_measurement())
                .count();
            let unitary_count = unitary_belongings.iter()
                .filter_map(|idx| get_gate_fragment_label(idx))
                .filter(|label| label.is_unitary())
                .count();
            let control_count = unitary_belongings.iter()
                .filter_map(|idx| get_gate_fragment_label(idx))
                .filter(|label| matches!(label, super::fragments::gate_fragments::GateFragmentLabel::Control(_)))
                .count();
                
            // Groups can be:
            // 1. Pure measurement groups (measurement_count > 0, unitary_count == 0, control_count == 0)
            // 2. Pure unitary groups (unitary_count > 0, measurement_count == 0)
            // 3. Mixed unitary+control groups (unitary_count > 0, control_count >= 0, measurement_count == 0)
            
            let is_pure_measurement = measurement_count > 0 && unitary_count == 0 && control_count == 0;
            let is_unitary_group = unitary_count > 0 && measurement_count == 0;
            
            if !is_pure_measurement && !is_unitary_group {
                return Err(format!("The group {} mixes measurements with unitary operations or has invalid composition", unitary_id));
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
        // QuantumCircuit is a sequence of GateActions. When a GateAction contains MeasurementDependence,
        // all GateActions referenced in the MeasurementDependence must appear before the current GateAction.
        // This ensures proper temporal ordering of measurement-dependent operations.
        
        self.check_all_uniatry()?;
        let toposorted = self.into_dependency_graph().toposort()?;
        
        // Create a mapping from gate fragment ID to GateAction index
        let mut gate_action_indices: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
        let mut gate_actions = Vec::new();
        let mut measurement_gate_indices: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
        
        // First, process standalone measurement gates to assign them indices
        for gate_fragment in &self.gate_fragments {
            if gate_fragment.is_measurement() {
                let gate = super::circuits::Gate::new("measurement".to_string(), 1, true);
                let qubits = vec![gate_fragment.id % 100]; // Simple qubit mapping
                let gate_action = super::circuits::GateAction::new(gate, qubits, None);
                
                let measurement_index = gate_actions.len();
                measurement_gate_indices.insert(gate_fragment.id, measurement_index);
                gate_action_indices.insert(gate_fragment.id, measurement_index);
                gate_actions.push(gate_action);
            }
        }
        
        // Collect all unitary groups, including those not in the dependency graph
        let mut processed_groups = std::collections::HashSet::new();
        
        // Process each unitary group in topological order
        for unitary_id in toposorted.iter() {
            processed_groups.insert(*unitary_id);
            let fragments = self.control_edges.get_same_group(*unitary_id);
            
            // Separate unitary and control fragments  
            let unitary_fragments: Vec<_> = fragments.iter()
                .filter_map(|idx| {
                    if *idx < self.gate_fragments.len() {
                        let gf = &self.gate_fragments[*idx];
                        if gf.label.is_unitary() { Some(gf.clone()) } else { None }
                    } else { None }
                })
                .collect();
            
            let control_fragments: Vec<_> = fragments.iter()
                .filter_map(|idx| {
                    if *idx < self.gate_fragments.len() {
                        let gf = &self.gate_fragments[*idx];
                        match &gf.label {
                            super::fragments::gate_fragments::GateFragmentLabel::Control(_) => Some(gf.clone()),
                            _ => None
                        }
                    } else { None }
                })
                .collect();
            
            // Find measurement dependencies for this unitary group
            let mut dependent_measurement_ids = Vec::new();
            let mut measurement_label = None;
            
            // Check feedback edges to find measurement dependencies
            for (measurement_gates, targets_and_labels) in &self.feedback_edges.edges {
                for (target_gate_id, label) in targets_and_labels {
                    if fragments.contains(target_gate_id) {
                        // This unitary group depends on measurements
                        measurement_label = Some(label.clone());
                        for measurement_gate_id in measurement_gates {
                            if let Some(&measurement_index) = measurement_gate_indices.get(measurement_gate_id) {
                                dependent_measurement_ids.push(measurement_index);
                            }
                        }
                    }
                }
            }
            
            // Skip groups that don't contain unitary gates (they will be processed as standalone measurements)
            if unitary_fragments.is_empty() {
                continue;
            }
            
            // For now, assume each unitary group represents a single gate
            // In a more complex implementation, we'd need to handle multi-qubit gates
            let primary_unitary = &unitary_fragments[0];
            let gate_label = match &primary_unitary.label {
                super::fragments::gate_fragments::GateFragmentLabel::Unitary(u) => u.to_string(),
                _ => return Err("Expected unitary gate".to_string()),
            };
            
            // Determine if this is a controlled gate
            let is_controlled = !control_fragments.is_empty();
            let span_qubit_number = if is_controlled { 
                control_fragments.len() + unitary_fragments.len() 
            } else { 
                unitary_fragments.len() 
            };
            
            let gate = super::circuits::Gate::new(gate_label, span_qubit_number, false);
            
            // For this implementation, we'll use fragment IDs as qubit indices
            // In a real implementation, qubit mapping would be more sophisticated
            let mut qubits = Vec::new();
            for control_fragment in &control_fragments {
                qubits.push(control_fragment.id % 100); // Simple qubit mapping
            }
            for unitary_fragment in &unitary_fragments {
                qubits.push(unitary_fragment.id % 100); // Simple qubit mapping
            }
            qubits.sort();
            qubits.dedup();
            
            // Create measurement dependence if needed
            let measurement_dependence = if !dependent_measurement_ids.is_empty() {
                Some(super::circuits::MeasurementDependence {
                    label: measurement_label.unwrap_or_else(|| format!("dep_{}", gate_actions.len())),
                    dependent_measurement_ids,
                })
            } else {
                None
            };
            
            let gate_action = super::circuits::GateAction::new(gate, qubits, measurement_dependence);
            let action_index = gate_actions.len();
            gate_actions.push(gate_action);
            
            // Record the mapping for this unitary group
            for fragment_idx in &fragments {
                gate_action_indices.insert(*fragment_idx, action_index);
            }
        }
        
        // Process unitary groups that were not in the dependency graph (isolated gates)
        for (group_representative, group_fragments) in self.control_edges.get_groups() {
            if processed_groups.contains(&group_representative) {
                continue; // Already processed
            }
            
            // Check if this group contains any unitary gates
            let unitary_fragments: Vec<_> = group_fragments.iter()
                .filter_map(|idx| {
                    if *idx < self.gate_fragments.len() {
                        let gf = &self.gate_fragments[*idx];
                        if gf.label.is_unitary() { Some(gf.clone()) } else { None }
                    } else { None }
                })
                .collect();
                
            if unitary_fragments.is_empty() {
                continue; // No unitary gates in this group
            }
            
            let control_fragments: Vec<_> = group_fragments.iter()
                .filter_map(|idx| {
                    if *idx < self.gate_fragments.len() {
                        let gf = &self.gate_fragments[*idx];
                        match &gf.label {
                            super::fragments::gate_fragments::GateFragmentLabel::Control(_) => Some(gf.clone()),
                            _ => None
                        }
                    } else { None }
                })
                .collect();
            
            // Create the gate based on unitary fragments
            let primary_unitary = &unitary_fragments[0];
            let gate_label = match &primary_unitary.label {
                super::fragments::gate_fragments::GateFragmentLabel::Unitary(u) => u.to_string(),
                _ => continue,
            };
            
            // Determine if this is a controlled gate
            let is_controlled = !control_fragments.is_empty();
            let span_qubit_number = if is_controlled { 
                control_fragments.len() + unitary_fragments.len() 
            } else { 
                unitary_fragments.len() 
            };
            
            let gate = super::circuits::Gate::new(gate_label, span_qubit_number, false);
            
            // For this implementation, we'll use fragment IDs as qubit indices
            let mut qubits = Vec::new();
            for control_fragment in &control_fragments {
                qubits.push(control_fragment.id % 100); // Simple qubit mapping
            }
            for unitary_fragment in &unitary_fragments {
                qubits.push(unitary_fragment.id % 100); // Simple qubit mapping
            }
            qubits.sort();
            qubits.dedup();
            
            let gate_action = super::circuits::GateAction::new(gate, qubits, None);
            let action_index = gate_actions.len();
            gate_actions.push(gate_action);
            
            // Record the mapping for this unitary group
            for fragment_idx in &group_fragments {
                gate_action_indices.insert(*fragment_idx, action_index);
            }
        }
        
        // Calculate the total number of qubits
        let max_qubit = gate_actions.iter()
            .flat_map(|ga| &ga.qubits)
            .max()
            .copied()
            .unwrap_or(0);
        
        let quantum_circuit = super::circuits::QuantumCircuit {
            gates: gate_actions,
            qubits: max_qubit + 1,
        };
        
        Ok(quantum_circuit)
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

    #[test]
    fn test_try_into_simple_unitary() {
        use crate::fragments::{circuit_fragments::CircuitFragment, gate_fragments::{GateFragment, GateFragmentLabel, Unitary}};
        
        // Create a simple circuit fragment with one unitary gate
        let mut circuit_fragment = CircuitFragment::new(vec![]);
        let gate_fragment = GateFragment::new(GateFragmentLabel::Unitary(Unitary::H));
        circuit_fragment.add_gate_fragment(gate_fragment).unwrap();
        
        // Convert to quantum circuit
        let result: Result<QuantumCircuit, String> = circuit_fragment.try_into();
        assert!(result.is_ok());
        
        let quantum_circuit = result.unwrap();
        assert_eq!(quantum_circuit.gates.len(), 1);
        assert_eq!(quantum_circuit.gates[0].qubits.len(), 1);
        assert!(quantum_circuit.gates[0].measurement_dependence.is_none());
    }

    #[test]
    fn test_try_into_measurement_only() {
        use crate::fragments::{circuit_fragments::CircuitFragment, gate_fragments::{GateFragment, GateFragmentLabel}};
        
        // Create a circuit fragment with only measurement gates
        let mut circuit_fragment = CircuitFragment::new(vec![]);
        let measurement_fragment = GateFragment::new(GateFragmentLabel::Instrument);
        circuit_fragment.add_gate_fragment(measurement_fragment).unwrap();
        
        // Convert to quantum circuit
        let result: Result<QuantumCircuit, String> = circuit_fragment.try_into();
        if let Err(ref e) = result {
            eprintln!("Error: {}", e);
        }
        assert!(result.is_ok());
        
        let quantum_circuit = result.unwrap();
        assert_eq!(quantum_circuit.gates.len(), 1);
        assert!(quantum_circuit.gates[0].gate.is_measurement);
        assert!(quantum_circuit.gates[0].measurement_dependence.is_none());
    }

    #[test]
    fn test_try_into_controlled_gate() {
        use crate::fragments::{circuit_fragments::CircuitFragment, gate_fragments::{GateFragment, GateFragmentLabel, Unitary, Control}};
        
        // Create a circuit fragment with control and target gates
        let mut circuit_fragment = CircuitFragment::new(vec![]);
        
        let control_fragment = GateFragment::new(GateFragmentLabel::Control(Control::Truthy));
        let target_fragment = GateFragment::new(GateFragmentLabel::Unitary(Unitary::X));
        
        circuit_fragment.add_gate_fragment(control_fragment.clone()).unwrap();
        circuit_fragment.add_gate_fragment(target_fragment.clone()).unwrap();
        
        // Connect control and target
        circuit_fragment.connect_control_edges(0, 1).unwrap();
        
        // Convert to quantum circuit
        let result: Result<QuantumCircuit, String> = circuit_fragment.try_into();
        assert!(result.is_ok());
        
        let quantum_circuit = result.unwrap();
        assert_eq!(quantum_circuit.gates.len(), 1);
        assert_eq!(quantum_circuit.gates[0].qubits.len(), 2); // Control + target
        assert!(quantum_circuit.gates[0].measurement_dependence.is_none());
    }

    #[test]
    fn test_try_into_measurement_feedback() {
        use crate::fragments::{circuit_fragments::CircuitFragment, gate_fragments::{GateFragment, GateFragmentLabel, Unitary}};
        
        // Create a circuit fragment with measurement and feedback-dependent gate
        let mut circuit_fragment = CircuitFragment::new(vec![]);
        
        let measurement_fragment = GateFragment::new(GateFragmentLabel::Instrument);
        let dependent_gate = GateFragment::new(GateFragmentLabel::Unitary(Unitary::X));
        
        circuit_fragment.add_gate_fragment(measurement_fragment.clone()).unwrap();
        circuit_fragment.add_gate_fragment(dependent_gate.clone()).unwrap();
        
        // Connect measurement feedback to the dependent gate
        circuit_fragment.connect_feedback_edges(
            vec![measurement_fragment.id],
            1, // target gate index
            "feedback_1".to_string(),
        ).unwrap();
        
        // Convert to quantum circuit
        let result: Result<QuantumCircuit, String> = circuit_fragment.try_into();
        if let Err(ref e) = result {
            eprintln!("Error: {}", e);
        }
        assert!(result.is_ok());
        
        let quantum_circuit = result.unwrap();
        assert_eq!(quantum_circuit.gates.len(), 2);
        
        // First gate should be measurement
        assert!(quantum_circuit.gates[0].gate.is_measurement);
        assert!(quantum_circuit.gates[0].measurement_dependence.is_none());
        
        // Second gate should depend on the measurement
        assert!(!quantum_circuit.gates[1].gate.is_measurement);
        assert!(quantum_circuit.gates[1].measurement_dependence.is_some());
        
        let measurement_dep = quantum_circuit.gates[1].measurement_dependence.as_ref().unwrap();
        assert_eq!(measurement_dep.label, "feedback_1");
        assert_eq!(measurement_dep.dependent_measurement_ids, vec![0]);
    }

    #[test]
    fn test_try_into_multiple_measurement_dependencies() {
        use crate::fragments::{circuit_fragments::CircuitFragment, gate_fragments::{GateFragment, GateFragmentLabel, Unitary}};
        
        // Create a circuit fragment with multiple measurements and one dependent gate
        let mut circuit_fragment = CircuitFragment::new(vec![]);
        
        let measurement1 = GateFragment::new(GateFragmentLabel::Instrument);
        let measurement2 = GateFragment::new(GateFragmentLabel::Instrument);
        let dependent_gate = GateFragment::new(GateFragmentLabel::Unitary(Unitary::Y));
        
        circuit_fragment.add_gate_fragment(measurement1.clone()).unwrap();
        circuit_fragment.add_gate_fragment(measurement2.clone()).unwrap();
        circuit_fragment.add_gate_fragment(dependent_gate.clone()).unwrap();
        
        // Connect both measurements to the dependent gate
        circuit_fragment.connect_feedback_edges(
            vec![measurement1.id, measurement2.id],
            2, // target gate index
            "multi_feedback".to_string(),
        ).unwrap();
        
        // Convert to quantum circuit
        let result: Result<QuantumCircuit, String> = circuit_fragment.try_into();
        assert!(result.is_ok());
        
        let quantum_circuit = result.unwrap();
        assert_eq!(quantum_circuit.gates.len(), 3);
        
        // First two gates should be measurements
        assert!(quantum_circuit.gates[0].gate.is_measurement);
        assert!(quantum_circuit.gates[1].gate.is_measurement);
        
        // Third gate should depend on both measurements
        assert!(!quantum_circuit.gates[2].gate.is_measurement);
        assert!(quantum_circuit.gates[2].measurement_dependence.is_some());
        
        let measurement_dep = quantum_circuit.gates[2].measurement_dependence.as_ref().unwrap();
        assert_eq!(measurement_dep.label, "multi_feedback");
        assert_eq!(measurement_dep.dependent_measurement_ids.len(), 2);
        assert!(measurement_dep.dependent_measurement_ids.contains(&0));
        assert!(measurement_dep.dependent_measurement_ids.contains(&1));
    }

    #[test]
    fn test_try_into_measurement_dependency_validation() {
        use crate::fragments::{circuit_fragments::CircuitFragment, gate_fragments::{GateFragment, GateFragmentLabel, Unitary}};
        
        // Create a valid circuit with proper measurement dependency ordering
        let mut circuit_fragment = CircuitFragment::new(vec![]);
        
        let measurement = GateFragment::new(GateFragmentLabel::Instrument);
        let dependent_gate = GateFragment::new(GateFragmentLabel::Unitary(Unitary::Z));
        
        circuit_fragment.add_gate_fragment(measurement.clone()).unwrap();
        circuit_fragment.add_gate_fragment(dependent_gate.clone()).unwrap();
        
        circuit_fragment.connect_feedback_edges(
            vec![measurement.id],
            1,
            "test_feedback".to_string(),
        ).unwrap();
        
        // Convert to quantum circuit and validate
        let result: Result<QuantumCircuit, String> = circuit_fragment.try_into();
        assert!(result.is_ok());
        
        let quantum_circuit = result.unwrap();
        
        // Test the validation function
        quantum_circuit.validation(); // Should not panic
        
        // Verify that measurement comes before dependent gate
        assert!(quantum_circuit.gates[0].gate.is_measurement);
        assert!(!quantum_circuit.gates[1].gate.is_measurement);
        
        let measurement_dep = quantum_circuit.gates[1].measurement_dependence.as_ref().unwrap();
        for &dep_id in &measurement_dep.dependent_measurement_ids {
            assert!(dep_id < 1, "Measurement dependency must reference earlier gates");
        }
    }

    #[test]
    fn test_try_into_complex_circuit() {
        use crate::fragments::{circuit_fragments::CircuitFragment, gate_fragments::{GateFragment, GateFragmentLabel, Unitary, Control}};
        
        // Create a complex circuit with measurements, controls, and feedback
        let mut circuit_fragment = CircuitFragment::new(vec![]);
        
        // Add initial measurement
        let initial_measurement = GateFragment::new(GateFragmentLabel::Instrument);
        circuit_fragment.add_gate_fragment(initial_measurement.clone()).unwrap();
        
        // Add controlled gate that depends on measurement
        let control = GateFragment::new(GateFragmentLabel::Control(Control::Truthy));
        let target = GateFragment::new(GateFragmentLabel::Unitary(Unitary::H));
        circuit_fragment.add_gate_fragment(control.clone()).unwrap();
        circuit_fragment.add_gate_fragment(target.clone()).unwrap();
        
        // Connect control and target
        circuit_fragment.connect_control_edges(1, 2).unwrap();
        
        // Add feedback from measurement to controlled gate
        circuit_fragment.connect_feedback_edges(
            vec![initial_measurement.id],
            1, // control fragment index
            "complex_feedback".to_string(),
        ).unwrap();
        
        // Add final measurement
        let final_measurement = GateFragment::new(GateFragmentLabel::Instrument);
        circuit_fragment.add_gate_fragment(final_measurement.clone()).unwrap();
        
        // Convert to quantum circuit
        let result: Result<QuantumCircuit, String> = circuit_fragment.try_into();
        assert!(result.is_ok());
        
        let quantum_circuit = result.unwrap();
        
        // Should have initial measurement, controlled gate, and final measurement
        assert_eq!(quantum_circuit.gates.len(), 3);
        
        // Validate the circuit structure
        quantum_circuit.validation(); // Should not panic
        
        // Check that feedback dependency is properly ordered
        let controlled_gate_action = quantum_circuit.gates.iter()
            .find(|ga| !ga.gate.is_measurement && ga.measurement_dependence.is_some())
            .expect("Should have a gate with measurement dependence");
        
        assert_eq!(controlled_gate_action.measurement_dependence.as_ref().unwrap().label, "complex_feedback");
    }
}