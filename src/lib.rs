pub mod gates;
pub mod primitive;
pub mod pyfunctions;
pub mod pyzx;
pub mod qasm;
pub mod select_gates;
pub mod util;

use pyo3::prelude::*;

use pyfunctions::{
    dist_select_simple::dist_select_simple_export,
    json_based::{count_t_depth, layered, uniform_layered},
    m_body::uniform_layered_m_body,
    qasm_layered::{uniform_layered_qasm, uniform_layered_redundant},
    tests::{output_json, sum_as_string, test_gate, test_gate_qasm},
};

/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
fn prepare_circuit(_py: Python, m: &PyModule) -> PyResult<()> {
    println!("prepare-circuit version 1.0.10");
    // tests
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_function(wrap_pyfunction!(output_json, m)?)?;
    m.add_function(wrap_pyfunction!(test_gate, m)?)?;
    m.add_function(wrap_pyfunction!(test_gate_qasm, m)?)?;
    // json_based
    m.add_function(wrap_pyfunction!(layered, m)?)?;
    m.add_function(wrap_pyfunction!(count_t_depth, m)?)?;
    m.add_function(wrap_pyfunction!(uniform_layered, m)?)?;
    // qasm_layerd
    m.add_function(wrap_pyfunction!(uniform_layered_qasm, m)?)?;
    m.add_function(wrap_pyfunction!(uniform_layered_redundant, m)?)?;
    // dist_select_simple
    m.add_function(wrap_pyfunction!(dist_select_simple_export, m)?)?;
    // m_body
    m.add_function(wrap_pyfunction!(uniform_layered_m_body, m)?)?;
    Ok(())
}
