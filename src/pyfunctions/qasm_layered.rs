use crate::qasm::to_qasm::*;

use super::internal::{uniform_layered_internal, uniform_layered_internal_redundant};

use pyo3::prelude::*;

/// generates uniform layered qrom in qasm format
#[pyfunction]
#[pyo3(text_signature = "(n, count, /)")]
pub fn uniform_layered_qasm(n: i32, count: i32) -> PyResult<Vec<String>> {
    let qubits_vec = uniform_layered_internal(n, count);

    let qasms = qubits_vec
        .iter()
        .map(|qubits| {
            let qasm_file = to_qasm(qubits.clone());
            qasm_file.to_string()
        })
        .collect::<Vec<_>>();

    Ok(qasms)
}

#[pyfunction]
#[pyo3(text_signature = "(n, count, r, /)")]
pub fn uniform_layered_redundant(n: i32, count: i32, r: i32) -> PyResult<Vec<String>> {
    let qubits_vec = uniform_layered_internal_redundant(n, count, r);

    let qasms = qubits_vec
        .iter()
        .map(|qubits| {
            let qasm_file = to_qasm(qubits.clone());
            qasm_file.to_string()
        })
        .collect::<Vec<_>>();

    Ok(qasms)
}
