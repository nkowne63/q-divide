use std::{collections::HashMap, result::Result};
use super::{circuit_fragments::CircuitFragment, gate_fragments::{GateFragment, GateFragmentLabel, Unitary}};

struct Gate {
    gate_type: Unitary
}

impl Into<GateFragment> for &Gate {
    fn into(self) -> GateFragment {
        let label = GateFragmentLabel::Unitary(self.gate_type.clone());
        GateFragment::new(label)
    }
}

struct DanglingFeedback {
    measurement: Vec<usize>,
    label: String
}

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
        todo!(); // add control
    }
    fn measure(&mut self, qubits: Vec<usize>, label: String) -> DanglingFeedback {
        todo!(); // add measure
    }
    fn control_connect(&mut self, edges: (&DanglingControl, DanglingTarget)) -> Result<DanglingTarget, String> {
        todo!(); // add connect + enable 1 control M target
    }
    fn feedback_connect(&mut self, edges: (DanglingFeedback, DanglingTarget)) -> Result<(), String> {
        todo!(); // add connect
    }
}