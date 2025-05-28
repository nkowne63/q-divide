use std::{collections::HashMap, result::Result};
use super::{circuit_fragments::CircuitFragment, gate_fragments::{GateFragment, GateFragmentLabel, Unitary, Control}};

struct Gate {
    gate_type: Unitary
}

impl Into<GateFragment> for &Gate {
    fn into(self) -> GateFragment {
        let label = GateFragmentLabel::Unitary(self.gate_type.clone());
        GateFragment::new(label)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct DanglingFeedback {
    measurement: Vec<usize>,
    label: String
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
struct DanglingTarget {
    gate_fragment: usize
}

#[derive(Debug, Clone, Copy)]
struct DanglingControl {
    gate_fragment: usize
}

struct QubitWiseFragmentInfo {
    initial: usize,
    latest: usize
}

struct CircuitLike {
    fragment: CircuitFragment,
    qubitwise_fragments: HashMap<usize, QubitWiseFragmentInfo>,
}

impl CircuitLike {
    fn new() -> Self {
        CircuitLike {
            fragment: CircuitFragment::new(vec![]),
            qubitwise_fragments: HashMap::new(),
        }
    }
    fn sq_gate(&mut self, qubit: usize, gate: &Gate) -> Result<DanglingTarget, String> {
        let gate_fragment: GateFragment = gate.into();
        self.fragment.add_gate_fragment(gate_fragment.clone())?;
        let qubitwise_fragment_info = self.qubitwise_fragments.get(&qubit);
        match qubitwise_fragment_info {
            None => {
                self.qubitwise_fragments.insert(qubit, QubitWiseFragmentInfo { initial: gate_fragment.id, latest: gate_fragment.id });
            }
            Some(info) => {
                let latest = info.latest;
                self.fragment.connect_qubit_edges(latest, gate_fragment.id)?;
                self.qubitwise_fragments.insert(qubit, QubitWiseFragmentInfo { initial: info.initial, latest: gate_fragment.id });
            }
        }
        Ok(DanglingTarget { gate_fragment: gate_fragment.id })
    }
    fn control(&mut self, control_qubit: usize) -> DanglingControl {
        // create a dangling control gate fragment and add to fragment then return DanglingControl
        let control_gate_fragment = GateFragment::new(GateFragmentLabel::Control(Control::Truthy));
        let gate_id = control_gate_fragment.id;
        
        // Add the control gate fragment to the circuit
        self.fragment.add_gate_fragment(control_gate_fragment).expect("Failed to add control gate fragment");
        
        // Update qubitwise fragment information for the control qubit
        let qubitwise_fragment_info = self.qubitwise_fragments.get(&control_qubit);
        match qubitwise_fragment_info {
            None => {
                // First gate on this qubit
                self.qubitwise_fragments.insert(control_qubit, QubitWiseFragmentInfo { 
                    initial: gate_id, 
                    latest: gate_id 
                });
            }
            Some(info) => {
                // Connect to the previous gate on this qubit
                let latest = info.latest;
                self.fragment.connect_qubit_edges(latest, gate_id).expect("Failed to connect qubit edges");
                self.qubitwise_fragments.insert(control_qubit, QubitWiseFragmentInfo { 
                    initial: info.initial, 
                    latest: gate_id 
                });
            }
        }
        
        DanglingControl { gate_fragment: gate_id }
    }
    fn measure(&mut self, qubits: Vec<usize>, label: String) -> DanglingFeedback {
        // create measurement gate fragments for each qubit and add to circuit fragment
        let mut measurement_gates = Vec::new();
        
        for &qubit in &qubits {
            // Create measurement gate fragment
            let measurement_gate_fragment = GateFragment::new(GateFragmentLabel::Instrument);
            let gate_id = measurement_gate_fragment.id;
            measurement_gates.push(gate_id);
            
            // Add the measurement gate fragment to the circuit
            self.fragment.add_gate_fragment(measurement_gate_fragment).expect("Failed to add measurement gate fragment");
            
            // Update qubitwise fragment information for the measured qubit
            let qubitwise_fragment_info = self.qubitwise_fragments.get(&qubit);
            match qubitwise_fragment_info {
                None => {
                    // First gate on this qubit
                    self.qubitwise_fragments.insert(qubit, QubitWiseFragmentInfo { 
                        initial: gate_id, 
                        latest: gate_id 
                    });
                }
                Some(info) => {
                    // Connect to the previous gate on this qubit
                    let latest = info.latest;
                    self.fragment.connect_qubit_edges(latest, gate_id).expect("Failed to connect qubit edges");
                    self.qubitwise_fragments.insert(qubit, QubitWiseFragmentInfo { 
                        initial: info.initial, 
                        latest: gate_id 
                    });
                }
            }
        }
        
        DanglingFeedback { 
            measurement: measurement_gates,
            label 
        }
    }
    fn control_connect(&mut self, edges: (&DanglingControl, DanglingTarget)) -> Result<DanglingTarget, String> {
        let (control, target) = edges;
        
        // Find the indices of the control and target gate fragments
        let control_index = self.fragment.gate_fragments
            .iter()
            .position(|g| g.id == control.gate_fragment)
            .ok_or(format!("Control gate fragment with id {} not found", control.gate_fragment))?;
            
        let target_index = self.fragment.gate_fragments
            .iter()
            .position(|g| g.id == target.gate_fragment)
            .ok_or(format!("Target gate fragment with id {} not found", target.gate_fragment))?;
        
        // Connect the control and target gates using control edges
        self.fragment.connect_control_edges(control_index, target_index)?;
        
        // Return the same target to allow chaining
        Ok(DanglingTarget { gate_fragment: target.gate_fragment })
    }
    fn feedback_connect(&mut self, edges: (DanglingFeedback, DanglingTarget)) -> Result<(), String> {
        let (feedback, target) = edges;
        
        // Find the index of the target gate fragment
        let target_index = self.fragment.gate_fragments
            .iter()
            .position(|g| g.id == target.gate_fragment)
            .ok_or(format!("Target gate fragment with id {} not found", target.gate_fragment))?;
        
        // Verify all measurement gate fragments exist
        for &measurement_id in &feedback.measurement {
            self.fragment.gate_fragments
                .iter()
                .find(|g| g.id == measurement_id)
                .ok_or(format!("Measurement gate fragment with id {} not found", measurement_id))?;
        }
        
        // Connect the feedback measurements to the target using feedback edges
        self.fragment.connect_feedback_edges(feedback.measurement, target_index, feedback.label)?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_control_creates_control_gate_fragment() {
        let mut circuit = CircuitLike::new();
        let control_qubit = 0;
        
        let dangling_control = circuit.control(control_qubit);
        
        // Verify that a gate fragment was added to the circuit
        assert_eq!(circuit.fragment.gate_fragments.len(), 1);
        
        // Verify that the gate fragment is a control gate
        let gate_fragment = &circuit.fragment.gate_fragments[0];
        assert_eq!(gate_fragment.id, dangling_control.gate_fragment);
        match &gate_fragment.label {
            GateFragmentLabel::Control(Control::Truthy) => {},
            _ => panic!("Expected Control gate fragment"),
        }
    }

    #[test]
    fn test_control_updates_qubitwise_fragments_first_gate() {
        let mut circuit = CircuitLike::new();
        let control_qubit = 0;
        
        let dangling_control = circuit.control(control_qubit);
        
        // Verify qubitwise fragment information is updated
        let qubit_info = circuit.qubitwise_fragments.get(&control_qubit).unwrap();
        assert_eq!(qubit_info.initial, dangling_control.gate_fragment);
        assert_eq!(qubit_info.latest, dangling_control.gate_fragment);
    }

    #[test]
    fn test_control_connects_to_previous_gate_on_qubit() {
        let mut circuit = CircuitLike::new();
        let control_qubit = 0;
        let gate = Gate { gate_type: Unitary::H };
        
        // Add a gate first
        let _target = circuit.sq_gate(control_qubit, &gate).unwrap();
        
        // Then add a control
        let dangling_control = circuit.control(control_qubit);
        
        // Verify two gate fragments exist
        assert_eq!(circuit.fragment.gate_fragments.len(), 2);
        
        // Verify qubitwise fragment information is updated
        let qubit_info = circuit.qubitwise_fragments.get(&control_qubit).unwrap();
        assert_eq!(qubit_info.latest, dangling_control.gate_fragment);
        
        // Verify that the control gate is connected to the previous gate
        let first_gate_id = circuit.fragment.gate_fragments[0].id;
        let control_gate_id = circuit.fragment.gate_fragments[1].id;
        assert_eq!(circuit.fragment.qubit_edges.edges.get(&first_gate_id), Some(&control_gate_id));
    }

    #[test]
    fn test_control_multiple_qubits() {
        let mut circuit = CircuitLike::new();
        
        let control1 = circuit.control(0);
        let control2 = circuit.control(1);
        
        // Verify both controls are different
        assert_ne!(control1.gate_fragment, control2.gate_fragment);
        
        // Verify both qubits have their own fragment info
        assert!(circuit.qubitwise_fragments.contains_key(&0));
        assert!(circuit.qubitwise_fragments.contains_key(&1));
        
        // Verify two gate fragments exist
        assert_eq!(circuit.fragment.gate_fragments.len(), 2);
    }

    #[test]
    fn test_control_returns_correct_dangling_control() {
        let mut circuit = CircuitLike::new();
        let control_qubit = 0;
        
        let dangling_control = circuit.control(control_qubit);
        
        // Verify the returned DanglingControl points to the correct gate fragment
        let gate_fragment = &circuit.fragment.gate_fragments[0];
        assert_eq!(dangling_control.gate_fragment, gate_fragment.id);
    }

    #[test]
    fn test_measure_creates_measurement_gate_fragments() {
        let mut circuit = CircuitLike::new();
        let qubits = vec![0, 1];
        let label = "measurement_1".to_string();
        
        let dangling_feedback = circuit.measure(qubits.clone(), label.clone());
        
        // Verify that measurement gate fragments were added to the circuit
        assert_eq!(circuit.fragment.gate_fragments.len(), 2);
        
        // Verify that the gate fragments are measurement gates
        for (i, &qubit) in qubits.iter().enumerate() {
            let gate_fragment = &circuit.fragment.gate_fragments[i];
            assert_eq!(gate_fragment.id, dangling_feedback.measurement[i]);
            match &gate_fragment.label {
                GateFragmentLabel::Instrument => {},
                _ => panic!("Expected Instrument gate fragment for qubit {}", qubit),
            }
        }
        
        // Verify the label is correct
        assert_eq!(dangling_feedback.label, label);
    }

    #[test]
    fn test_measure_updates_qubitwise_fragments_first_gates() {
        let mut circuit = CircuitLike::new();
        let qubits = vec![0, 1];
        let label = "measurement_1".to_string();
        
        let dangling_feedback = circuit.measure(qubits.clone(), label);
        
        // Verify qubitwise fragment information is updated for each qubit
        for (i, &qubit) in qubits.iter().enumerate() {
            let qubit_info = circuit.qubitwise_fragments.get(&qubit).unwrap();
            assert_eq!(qubit_info.initial, dangling_feedback.measurement[i]);
            assert_eq!(qubit_info.latest, dangling_feedback.measurement[i]);
        }
    }

    #[test]
    fn test_measure_connects_to_previous_gates_on_qubits() {
        let mut circuit = CircuitLike::new();
        let qubits = vec![0, 1];
        let gate = Gate { gate_type: Unitary::H };
        let label = "measurement_1".to_string();
        
        // Add gates first
        let _target0 = circuit.sq_gate(0, &gate).unwrap();
        let _target1 = circuit.sq_gate(1, &gate).unwrap();
        
        // Then add measurements
        let dangling_feedback = circuit.measure(qubits.clone(), label);
        
        // Verify four gate fragments exist (2 H gates + 2 measurement gates)
        assert_eq!(circuit.fragment.gate_fragments.len(), 4);
        
        // Verify qubitwise fragment information is updated
        for (i, &qubit) in qubits.iter().enumerate() {
            let qubit_info = circuit.qubitwise_fragments.get(&qubit).unwrap();
            assert_eq!(qubit_info.latest, dangling_feedback.measurement[i]);
        }
        
        // Verify that measurement gates are connected to previous gates
        let h_gate0_id = circuit.fragment.gate_fragments[0].id;
        let h_gate1_id = circuit.fragment.gate_fragments[1].id;
        let measure_gate0_id = circuit.fragment.gate_fragments[2].id;
        let measure_gate1_id = circuit.fragment.gate_fragments[3].id;
        
        assert_eq!(circuit.fragment.qubit_edges.edges.get(&h_gate0_id), Some(&measure_gate0_id));
        assert_eq!(circuit.fragment.qubit_edges.edges.get(&h_gate1_id), Some(&measure_gate1_id));
    }

    #[test]
    fn test_measure_single_qubit() {
        let mut circuit = CircuitLike::new();
        let qubits = vec![0];
        let label = "single_measurement".to_string();
        
        let dangling_feedback = circuit.measure(qubits, label.clone());
        
        // Verify one measurement gate fragment was added
        assert_eq!(circuit.fragment.gate_fragments.len(), 1);
        assert_eq!(dangling_feedback.measurement.len(), 1);
        
        // Verify the gate fragment is a measurement gate
        let gate_fragment = &circuit.fragment.gate_fragments[0];
        assert_eq!(gate_fragment.id, dangling_feedback.measurement[0]);
        match &gate_fragment.label {
            GateFragmentLabel::Instrument => {},
            _ => panic!("Expected Instrument gate fragment"),
        }
        
        // Verify the label
        assert_eq!(dangling_feedback.label, label);
    }

    #[test]
    fn test_measure_empty_qubits() {
        let mut circuit = CircuitLike::new();
        let qubits = vec![];
        let label = "empty_measurement".to_string();
        
        let dangling_feedback = circuit.measure(qubits, label.clone());
        
        // Verify no gate fragments were added
        assert_eq!(circuit.fragment.gate_fragments.len(), 0);
        assert_eq!(dangling_feedback.measurement.len(), 0);
        
        // Verify the label
        assert_eq!(dangling_feedback.label, label);
    }

    #[test]
    fn test_measure_returns_correct_dangling_feedback() {
        let mut circuit = CircuitLike::new();
        let qubits = vec![0, 1];
        let label = "test_measurement".to_string();
        
        let dangling_feedback = circuit.measure(qubits.clone(), label.clone());
        
        // Verify the returned DanglingFeedback contains correct measurement gate IDs
        assert_eq!(dangling_feedback.measurement.len(), qubits.len());
        for (i, &expected_id) in dangling_feedback.measurement.iter().enumerate() {
            let gate_fragment = &circuit.fragment.gate_fragments[i];
            assert_eq!(gate_fragment.id, expected_id);
        }
        
        // Verify the label
        assert_eq!(dangling_feedback.label, label);
    }

    #[test]
    fn test_control_connect_basic() {
        let mut circuit = CircuitLike::new();
        let gate = Gate { gate_type: Unitary::X };
        
        // Create a control and a target
        let control = circuit.control(0);
        let target = circuit.sq_gate(1, &gate).unwrap();
        
        // Connect them
        let result = circuit.control_connect((&control, target));
        assert!(result.is_ok());
        
        // Verify they are in the same control group
        let control_index = circuit.fragment.gate_fragments
            .iter()
            .position(|g| g.id == control.gate_fragment)
            .unwrap();
        let target_index = circuit.fragment.gate_fragments
            .iter()
            .position(|g| g.id == target.gate_fragment)
            .unwrap();
        
        assert!(circuit.fragment.control_edges.is_same_set(control_index, target_index));
    }

    #[test]
    fn test_control_connect_returns_target() {
        let mut circuit = CircuitLike::new();
        let gate = Gate { gate_type: Unitary::X };
        
        // Create a control and a target
        let control = circuit.control(0);
        let target = circuit.sq_gate(1, &gate).unwrap();
        
        // Connect them and verify the returned target is the same
        let result = circuit.control_connect((&control, target)).unwrap();
        assert_eq!(result.gate_fragment, target.gate_fragment);
    }

    #[test]
    fn test_control_connect_one_control_multiple_targets() {
        let mut circuit = CircuitLike::new();
        let gate = Gate { gate_type: Unitary::X };
        
        // Create one control and multiple targets
        let control = circuit.control(0);
        let target1 = circuit.sq_gate(1, &gate).unwrap();
        let target2 = circuit.sq_gate(2, &gate).unwrap();
        
        // Connect control to both targets
        circuit.control_connect((&control, target1)).unwrap();
        circuit.control_connect((&control, target2)).unwrap();
        
        // Find indices
        let control_index = circuit.fragment.gate_fragments
            .iter()
            .position(|g| g.id == control.gate_fragment)
            .unwrap();
        let target1_index = circuit.fragment.gate_fragments
            .iter()
            .position(|g| g.id == target1.gate_fragment)
            .unwrap();
        let target2_index = circuit.fragment.gate_fragments
            .iter()
            .position(|g| g.id == target2.gate_fragment)
            .unwrap();
        
        // Verify all three are in the same control group
        assert!(circuit.fragment.control_edges.is_same_set(control_index, target1_index));
        assert!(circuit.fragment.control_edges.is_same_set(control_index, target2_index));
        assert!(circuit.fragment.control_edges.is_same_set(target1_index, target2_index));
    }

    #[test]
    fn test_control_connect_multiple_controls_same_target() {
        let mut circuit = CircuitLike::new();
        let gate = Gate { gate_type: Unitary::X };
        
        // Create multiple controls and one target
        let control1 = circuit.control(0);
        let control2 = circuit.control(1);
        let target = circuit.sq_gate(2, &gate).unwrap();
        
        // Connect both controls to the target
        circuit.control_connect((&control1, target)).unwrap();
        circuit.control_connect((&control2, target)).unwrap();
        
        // Find indices
        let control1_index = circuit.fragment.gate_fragments
            .iter()
            .position(|g| g.id == control1.gate_fragment)
            .unwrap();
        let control2_index = circuit.fragment.gate_fragments
            .iter()
            .position(|g| g.id == control2.gate_fragment)
            .unwrap();
        let target_index = circuit.fragment.gate_fragments
            .iter()
            .position(|g| g.id == target.gate_fragment)
            .unwrap();
        
        // Verify all three are in the same control group
        assert!(circuit.fragment.control_edges.is_same_set(control1_index, target_index));
        assert!(circuit.fragment.control_edges.is_same_set(control2_index, target_index));
        assert!(circuit.fragment.control_edges.is_same_set(control1_index, control2_index));
    }

    #[test]
    fn test_control_connect_invalid_control_id() {
        let mut circuit = CircuitLike::new();
        let gate = Gate { gate_type: Unitary::X };
        
        // Create a target but use an invalid control
        let target = circuit.sq_gate(1, &gate).unwrap();
        let invalid_control = DanglingControl { gate_fragment: 999 };
        
        // Attempt to connect should fail
        let result = circuit.control_connect((&invalid_control, target));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Control gate fragment with id 999 not found"));
    }

    #[test]
    fn test_control_connect_invalid_target_id() {
        let mut circuit = CircuitLike::new();
        
        // Create a control but use an invalid target
        let control = circuit.control(0);
        let invalid_target = DanglingTarget { gate_fragment: 999 };
        
        // Attempt to connect should fail
        let result = circuit.control_connect((&control, invalid_target));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Target gate fragment with id 999 not found"));
    }

    #[test]
    fn test_control_connect_chaining() {
        let mut circuit = CircuitLike::new();
        let gate = Gate { gate_type: Unitary::X };
        
        // Create control and targets
        let control = circuit.control(0);
        let target1 = circuit.sq_gate(1, &gate).unwrap();
        let target2 = circuit.sq_gate(2, &gate).unwrap();
        
        // Chain connections: control -> target1, then target1 result -> target2
        let connected_target1 = circuit.control_connect((&control, target1)).unwrap();
        let connected_target2 = circuit.control_connect((&control, target2)).unwrap();
        
        // Verify the returned targets are correct
        assert_eq!(connected_target1.gate_fragment, target1.gate_fragment);
        assert_eq!(connected_target2.gate_fragment, target2.gate_fragment);
    }

    #[test]
    fn test_feedback_connect_basic() {
        let mut circuit = CircuitLike::new();
        let gate = Gate { gate_type: Unitary::X };
        
        // Create measurements and a target
        let feedback = circuit.measure(vec![0, 1], "measurement_1".to_string());
        let target = circuit.sq_gate(2, &gate).unwrap();
        
        // Connect feedback to target
        let result = circuit.feedback_connect((feedback, target));
        assert!(result.is_ok());
        
        // Verify the feedback edge was created
        let target_index = circuit.fragment.gate_fragments
            .iter()
            .position(|g| g.id == target.gate_fragment)
            .unwrap();
        
        assert!(circuit.fragment.feedback_edges.labels.contains_key("measurement_1"));
        assert_eq!(circuit.fragment.feedback_edges.labels.get("measurement_1"), Some(&vec![target_index]));
    }

    #[test]
    fn test_feedback_connect_single_measurement() {
        let mut circuit = CircuitLike::new();
        let gate = Gate { gate_type: Unitary::H };
        
        // Create single measurement and target
        let feedback = circuit.measure(vec![0], "single_measure".to_string());
        let target = circuit.sq_gate(1, &gate).unwrap();
        
        // Connect feedback to target
        let result = circuit.feedback_connect((feedback, target));
        assert!(result.is_ok());
        
        // Verify the connection
        assert!(circuit.fragment.feedback_edges.labels.contains_key("single_measure"));
    }

    #[test]
    fn test_feedback_connect_multiple_measurements_same_target() {
        let mut circuit = CircuitLike::new();
        let gate = Gate { gate_type: Unitary::Y };
        
        // Create multiple measurements with different labels
        let feedback1 = circuit.measure(vec![0], "measure_1".to_string());
        let feedback2 = circuit.measure(vec![1], "measure_2".to_string());
        let target = circuit.sq_gate(2, &gate).unwrap();
        
        // Connect both feedbacks to the same target
        let result1 = circuit.feedback_connect((feedback1, target));
        let result2 = circuit.feedback_connect((feedback2, target));
        
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        
        // Verify both labels exist
        assert!(circuit.fragment.feedback_edges.labels.contains_key("measure_1"));
        assert!(circuit.fragment.feedback_edges.labels.contains_key("measure_2"));
    }

    #[test]
    fn test_feedback_connect_same_label_twice_fails() {
        let mut circuit = CircuitLike::new();
        let gate = Gate { gate_type: Unitary::Z };
        
        // Create feedback and targets
        let feedback1 = circuit.measure(vec![0], "duplicate_label".to_string());
        let feedback2 = circuit.measure(vec![1], "duplicate_label".to_string());
        let target1 = circuit.sq_gate(2, &gate).unwrap();
        let target2 = circuit.sq_gate(3, &gate).unwrap();
        
        // First connection should succeed
        let result1 = circuit.feedback_connect((feedback1, target1));
        assert!(result1.is_ok());
        
        // Second connection with same label should fail
        let result2 = circuit.feedback_connect((feedback2, target2));
        assert!(result2.is_err());
        assert!(result2.unwrap_err().contains("Label duplicate_label is already in use"));
    }

    #[test]
    fn test_feedback_connect_invalid_target_id() {
        let mut circuit = CircuitLike::new();
        
        // Create feedback with valid measurements but invalid target
        let feedback = circuit.measure(vec![0], "test_measure".to_string());
        let invalid_target = DanglingTarget { gate_fragment: 999 };
        
        // Connection should fail
        let result = circuit.feedback_connect((feedback, invalid_target));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Target gate fragment with id 999 not found"));
    }

    #[test]
    fn test_feedback_connect_invalid_measurement_id() {
        let mut circuit = CircuitLike::new();
        let gate = Gate { gate_type: Unitary::X };
        
        // Create target and invalid feedback
        let target = circuit.sq_gate(0, &gate).unwrap();
        let invalid_feedback = DanglingFeedback {
            measurement: vec![999],
            label: "invalid_measure".to_string()
        };
        
        // Connection should fail
        let result = circuit.feedback_connect((invalid_feedback, target));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Measurement gate fragment with id 999 not found"));
    }

    #[test]
    fn test_feedback_connect_empty_measurements() {
        let mut circuit = CircuitLike::new();
        let gate = Gate { gate_type: Unitary::H };
        
        // Create empty measurement and target
        let feedback = circuit.measure(vec![], "empty_measure".to_string());
        let target = circuit.sq_gate(0, &gate).unwrap();
        
        // Connection should succeed
        let result = circuit.feedback_connect((feedback, target));
        assert!(result.is_ok());
        
        // Verify the label exists
        assert!(circuit.fragment.feedback_edges.labels.contains_key("empty_measure"));
    }

    #[test]
    fn test_feedback_connect_multiple_qubits_feedback() {
        let mut circuit = CircuitLike::new();
        let gate = Gate { gate_type: Unitary::X };
        
        // Create multi-qubit measurement and target
        let feedback = circuit.measure(vec![0, 1, 2], "multi_qubit_measure".to_string());
        let target = circuit.sq_gate(3, &gate).unwrap();
        
        // Connect feedback to target
        let result = circuit.feedback_connect((feedback, target));
        assert!(result.is_ok());
        
        // Verify the connection
        let target_index = circuit.fragment.gate_fragments
            .iter()
            .position(|g| g.id == target.gate_fragment)
            .unwrap();
        
        assert!(circuit.fragment.feedback_edges.labels.contains_key("multi_qubit_measure"));
        assert_eq!(circuit.fragment.feedback_edges.labels.get("multi_qubit_measure"), Some(&vec![target_index]));
    }
}