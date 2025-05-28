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
struct DanglingFeedback {
    measurement: Vec<usize>,
    label: String
}

#[allow(dead_code)]
struct DanglingTarget {
    gate_fragment: usize
}

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
    #[allow(dead_code)]
    fn measure(&mut self, _qubits: Vec<usize>, _label: String) -> DanglingFeedback {
        todo!(); // add measure
    }
    #[allow(dead_code)]
    fn control_connect(&mut self, _edges: (&DanglingControl, DanglingTarget)) -> Result<DanglingTarget, String> {
        todo!(); // add connect + enable 1 control M target
    }
    #[allow(dead_code)]
    fn feedback_connect(&mut self, _edges: (DanglingFeedback, DanglingTarget)) -> Result<(), String> {
        todo!(); // add connect
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
}