use std::fmt::{Display, Formatter};

use crate::circuits::QuantumCircuit;

#[allow(dead_code)]
enum Gate {
    Single(usize),
    Double(usize, usize),
    Measurement(usize),
    Reset(usize),
    If(String, Vec<Gate>)
}

#[allow(dead_code)]
struct OpenQasmCircuit {
    qubits_count: usize,
    classical_bits_count: usize,
    dependent_functions: Vec<String>,
    gates: Vec<Gate>,
}

impl From<QuantumCircuit> for OpenQasmCircuit {
    fn from(_circuit: QuantumCircuit) -> Self {
        todo!() // conversion
    }
}

impl Display for OpenQasmCircuit {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!(); // string output
    }
}