use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Gate {
    pub label: String,
    pub span_qubit_number: usize,
    pub is_measurement: bool
}

impl Gate {
    pub fn new(label: String, span_qubit_number: usize, is_measurement: bool) -> Self {
        assert!(span_qubit_number > 0, "Gate span qubit number must be greater than 0");
        Gate {
            label,
            span_qubit_number,
            is_measurement
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MeasurementDependence {
    pub label: String,
    pub dependent_measurement_ids: Vec<usize>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GateAction {
    pub gate: Gate,
    pub qubits: Vec<usize>,
    pub measurement_dependence: Option<MeasurementDependence>
}

impl GateAction {
    pub fn new(gate: Gate, qubits: Vec<usize>, measurement_dependence: Option<MeasurementDependence>) -> Self {
        assert_eq!(gate.span_qubit_number, qubits.len(), "GateAction qubit count {} does not match gate span qubit number {}", gate.span_qubit_number, qubits.len());
        GateAction {
            gate,
            qubits,
            measurement_dependence
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuantumCircuit {
    pub gates: Vec<GateAction>,
    pub qubits: usize
}

impl QuantumCircuit {
    pub fn validation(&self) {
        for idx in 0..self.gates.len() {
            let gate = &self.gates[idx];
            if let Some(measurement_dependence) = &gate.measurement_dependence {
                for dependent_measurement_id in &measurement_dependence.dependent_measurement_ids {
                    assert!(*dependent_measurement_id < idx, "Measurement dependence must be on previous gates. Current gate index: {}, dependent measurement index: {}", idx, dependent_measurement_id);
                }
            }
        }
        let max_qubit_index = self.gates.iter().map(|gate| gate.qubits.iter().max().unwrap()).max().unwrap();
        assert_eq!(self.qubits, max_qubit_index + 1, "Qubit count {} does not match max qubit index {}", self.qubits, max_qubit_index);
    }
}