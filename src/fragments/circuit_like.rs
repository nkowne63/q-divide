use super::circuit_fragments::CircuitFragment;

struct Gate {
    gate_type: String
}

struct DanglingEdge {
    gate_fragment_index: usize
}

struct CircuitLike {
    fragment: CircuitFragment
}

impl CircuitLike {
    fn gate(&mut self, qubit: usize, gate: Gate) {
        todo!(); // add gate
    }
    fn control(&mut self, control_qubits: Vec<usize>) -> Vec<DanglingEdge> {
        todo!(); // add control
    }
    fn target(&mut self, target_qubits: Vec<usize>) -> Vec<DanglingEdge> {
        todo!(); // add target
    }
    fn measure(&mut self, qubit: usize, label: String) -> Vec<DanglingEdge> {
        todo!(); // add measure
    }
    fn connect(&mut self, edges: Vec<DanglingEdge>) -> Result<(), String> {
        todo!(); // add connect
    }
}