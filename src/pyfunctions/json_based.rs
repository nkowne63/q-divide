use crate::primitive::*;
use crate::pyzx::json::*;
use crate::pyzx::to_json::*;
use crate::select_gates::simple_select_controls::*;
use crate::util::*;

use super::internal::uniform_layered_internal;

use pyo3::prelude::*;

#[pyfunction]
pub fn uniform_layered(n: i32, count: i32) -> PyResult<Vec<String>> {
    let qubits_vec = uniform_layered_internal(n, count);

    let jsons = qubits_vec
        .iter()
        .map(|qubits| {
            let pyzx_json = to_pyzx_circuit(qubits.clone());
            serde_json::to_string(&pyzx_json).unwrap()
        })
        .collect::<Vec<_>>();

    Ok(jsons)
}

#[pyfunction]
#[pyo3(text_signature = "(json, /)")]
pub fn count_t_depth(json: String) -> PyResult<i32> {
    let pyzx: PyzxCircuitJson = serde_json::from_str(&json).unwrap();
    let plane = pyzx.produce_plane();
    let depth = PyzxCircuitJson::count_depth(&plane);
    Ok(depth)
}

#[pyfunction]
pub fn layered(n: i32) -> PyResult<String> {
    println!();
    let first_qubit = cellize(Qubit::new("first"));
    let first_control = Qubit::control(first_qubit.clone());
    let datas = (0..n)
        .map(|i| cellize(Qubit::new(format!("data_{}", i).as_str())))
        .collect::<Vec<_>>();
    let ancillas = (0..n)
        .map(|i| cellize(Qubit::new(format!("ancilla_{}", i).as_str())))
        .collect::<Vec<_>>();
    let controls = in_over_2n(n, &first_control, datas.clone(), ancillas.clone());
    let target_sample_1 = cellize(Qubit::new("target_s1"));
    let export_target_sample_1 = Qubit::export(target_sample_1.clone());
    export_target_sample_1.control_by(&controls[0]);
    let mut qubits = Vec::new();
    qubits.extend(datas);
    qubits.extend(ancillas);
    qubits.push(first_qubit);
    qubits.push(target_sample_1);

    let pyzx_json = to_pyzx_circuit(qubits);
    let json = serde_json::to_string(&pyzx_json).unwrap();
    Ok(json)
}
