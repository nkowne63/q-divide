use crate::gates::*;
use crate::primitive::*;
use crate::pyzx::json::*;
use crate::pyzx::to_json::*;
use crate::qasm::to_qasm::*;
use crate::util::*;

use pyo3::prelude::*;

/// Formats the sum of two numbers as string.
#[pyfunction]
pub fn sum_as_string(a: i32, b: i32) -> PyResult<String> {
    Ok((a + b + 1).to_string())
}

#[pyfunction]
pub fn test_gate() -> PyResult<String> {
    let q1 = cellize(Qubit::new("q1"));
    let q2 = cellize(Qubit::new("q2"));
    let q3 = cellize(Qubit::new("q3"));
    // let q4 = cellize(Qubit::new("q4"));
    // let control = Qubit::control(q1.clone());
    // let (leftc, _) = in_layer(&control, q2.clone(), q3.clone());
    // let leftc = in_layer(&control, q2.clone(), q3.clone());
    // let export = Qubit::export(q4.clone());
    // export.control_by(&leftc);
    toffoli(q1.clone(), q2.clone(), q3.clone());
    // let pyzx_json = to_pyzx_circuit(vec![q1.clone(), q2.clone(), q3.clone()]);
    // let pyzx_json = to_pyzx_circuit(vec![q1.clone(), q2.clone(), q3.clone(), q4.clone()]);
    let pyzx_json = to_pyzx_circuit(vec![q1, q2, q3]);
    let json = serde_json::to_string(&pyzx_json).unwrap();
    Ok(json)
}

#[pyfunction]
pub fn test_gate_qasm() -> PyResult<String> {
    let q1 = cellize(Qubit::new("q1"));
    let q2 = cellize(Qubit::new("q2"));
    let q3 = cellize(Qubit::new("q3"));
    // let q4 = cellize(Qubit::new("q4"));
    // let control = Qubit::control(q1.clone());
    // let (leftc, _) = in_layer(&control, q2.clone(), q3.clone());
    // let leftc = in_layer(&control, q2.clone(), q3.clone());
    // let export = Qubit::export(q4.clone());
    // export.control_by(&leftc);
    toffoli(q1.clone(), q2.clone(), q3.clone());
    // let pyzx_json = to_pyzx_circuit(vec![q1.clone(), q2.clone(), q3.clone()]);
    // let pyzx_json = to_pyzx_circuit(vec![q1.clone(), q2.clone(), q3.clone(), q4.clone()]);
    let qasm_file = to_qasm(vec![q1, q2, q3]);
    Ok(qasm_file.to_string())
}

#[pyfunction]
pub fn output_json() -> PyResult<String> {
    let mut wire_vertices = WireVertices::new();
    wire_vertices.insert(
        "b0".to_string(),
        WireVerticesValue {
            annotation: WireVerticesAnnotation {
                boundary: true,
                coord: vec![0.0, 0.0],
                input: true,
                output: false,
            },
        },
    );
    wire_vertices.insert(
        "b1".to_string(),
        WireVerticesValue {
            annotation: WireVerticesAnnotation {
                boundary: true,
                coord: vec![2.0, 0.0],
                input: false,
                output: true,
            },
        },
    );
    let mut node_vertices = NodeVertices::new();
    node_vertices.insert(
        "v0".to_string(),
        NodeVerticesValue {
            annotation: NodeVerticesAnnotation {
                coord: vec![1.0, 0.0],
            },
            data: NodeVerticesData {
                kind: "Z".to_string(),
                value: Some("\\pi".to_string()),
                is_edge: None,
            },
        },
    );
    let mut undir_edges = UndirEdges::new();
    undir_edges.insert(
        "e0".to_string(),
        UndirEdgesValue {
            src: "b0".to_string(),
            tgt: "v0".to_string(),
        },
    );
    undir_edges.insert(
        "e1".to_string(),
        UndirEdgesValue {
            src: "v0".to_string(),
            tgt: "b1".to_string(),
        },
    );
    let test_struct = PyzxCircuitJson {
        wire_vertices,
        node_vertices,
        undir_edges,
    };
    let serialized = serde_json::to_string(&test_struct).unwrap();
    Ok(serialized)
}
