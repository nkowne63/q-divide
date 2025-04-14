use std::fmt::{Display, Formatter};

use crate::circuits::QuantumCircuit;

enum Gate {
    Single(usize),
    Double(usize, usize),
    Measurement(usize),
    Reset(usize),
    If(String, Vec<Gate>)
}

struct OpenQasmCircuit {
    qubits_count: usize,
    classical_bits_count: usize,
    dependent_functions: Vec<String>,
    gates: Vec<Gate>,
}

// TODO: conversion
impl From<QuantumCircuit> for OpenQasmCircuit {
    fn from(circuit: QuantumCircuit) -> Self {
        todo!()
    }
}

// TODO: string output
impl Display for OpenQasmCircuit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "qasm")
    }
}