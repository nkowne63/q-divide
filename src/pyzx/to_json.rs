use super::json;
use super::serialize_utils::*;
use crate::primitive;

pub fn to_pyzx_graph(qubit_cells: Vec<primitive::QubitCell>) -> json::PyzxCircuitJson {
    let mut wire_vertices = json::WireVertices::new();
    let mut node_vertices = json::NodeVertices::new();
    let mut undir_edges = json::UndirEdges::new();
    let max_x = (qubit_cells
        .iter()
        .map(|q| q.operation_count())
        .max()
        .unwrap()
        + 1) as f64;
    qubit_cells.iter().enumerate().for_each(|(i, q)| {
        let y_coord = -(i as f64);
        // wire_verticesにまずは入れる
        wire_vertices.insert(
            q.input_qubit_id(),
            json::WireVerticesValue::create(true, vec![0.0, y_coord]),
        );
        wire_vertices.insert(
            q.output_qubit_id(),
            json::WireVerticesValue::create(false, vec![max_x, y_coord]),
        );
        // node_verticesを入れてx方向のundir_edgesを作る
        // controlled_notがあったら適宜y方向のundir_edgeを作る
        let operations = &q.as_ref().borrow().operations;
        let mut previous_node_id = q.input_qubit_id();
        let mut start_x_coord = 1.0;
        operations.iter().for_each(|op| {
            if !op.is_control() {
                // control以外のノードの場合
                // node_verticesを入れる
                let nodes = op.create_nodes(vec![start_x_coord, y_coord]);
                let (node_id, node) = nodes.get(0).unwrap();
                node_vertices.insert(node_id.clone(), node.clone());
                // 直前へのedgeを作る
                let op_id = op.get_node_id(None);
                undir_edges.insert(
                    format!("qedge_{}_op{}", previous_node_id, op_id),
                    json::UndirEdgesValue {
                        src: previous_node_id.clone(),
                        tgt: op_id.clone(),
                    },
                );
                start_x_coord += 1.0;
                previous_node_id = op_id;
            } else {
                // controlノードの場合（operationは一つだが、nodeがたくさんある）
                // nodeをたくさん入れる
                let nodes = op.create_nodes(vec![start_x_coord, y_coord]);
                nodes.iter().for_each(|(node_id, node)| {
                    node_vertices.insert(node_id.clone(), node.clone());
                    undir_edges.insert(
                        format!("qedge_{}_op{}", previous_node_id, node_id.clone()),
                        json::UndirEdgesValue {
                            src: previous_node_id.clone(),
                            tgt: node_id.clone(),
                        },
                    );
                    previous_node_id = node_id.clone();
                });
                start_x_coord += op.control_count() as f64;
            }
            if op.is_controlled() {
                // controlled_notの場合
                // controlへのedgeを張る
                let control_from_node_id = op.get_control_from_node_id();
                undir_edges.insert(
                    format!(
                        "qedge_control_{}_op{}",
                        control_from_node_id,
                        op.get_node_id(None)
                    ),
                    json::UndirEdgesValue {
                        src: control_from_node_id.clone(),
                        tgt: op.get_node_id(None).clone(),
                    },
                );
            }
        });
        // 最後だけは出力へのundir_edgeを連結する
        undir_edges.insert(
            format!("qedge_{}_last", q.qubit_id()),
            json::UndirEdgesValue {
                src: previous_node_id.clone(),
                tgt: q.output_qubit_id().clone(),
            },
        );
    });
    json::PyzxCircuitJson {
        wire_vertices,
        node_vertices,
        undir_edges,
    }
}

pub fn to_pyzx_circuit(qubit_cells: Vec<primitive::QubitCell>) -> json::PyzxCircuitJson {
    let mut wire_vertices = json::WireVertices::new();
    let mut node_vertices = json::NodeVertices::new();
    let mut undir_edges = json::UndirEdges::new();
    let mut current_x = 0 as f64;
    let iter = QubitsIter::new(qubit_cells.clone());
    // wire_verticesに最初のqubitを入れる
    qubit_cells.iter().enumerate().for_each(|(i, q)| {
        let y_coord = -(i as f64);
        wire_vertices.insert(
            q.input_qubit_id(),
            json::WireVerticesValue::create(true, vec![0.0, y_coord]),
        );
        // 最初のnode_idを取り出し、最初のqubitに連結する辺を張る
        let first_op = q.get_first();
        if first_op.is_some() {
            let first_op = first_op.unwrap();
            let is_cnot = first_op.is_control();
            let first_node_id = first_op.get_node_id(if is_cnot { Some(0) } else { None });
            undir_edges.insert(
                format!("qedge_{}_first", q.qubit_id()),
                json::UndirEdgesValue {
                    src: q.input_qubit_id().clone(),
                    tgt: first_node_id.clone(),
                },
            );
        }
    });
    // operationをiterしていく
    iter.enumerate().for_each(|(i, q)| {
        // println!("idx {:?}", q.clone());
        current_x = i as f64 + 1.0;
        let items = q.items;
        // itemsについてiter
        items.iter().enumerate().for_each(|(i, item)| {
            let y_coord = -(i as f64);
            match item {
                None => return,
                Some(item) => {
                    let QubitOperationsIterItem {
                        current,
                        previous,
                        cnot_pos,
                    } = item;
                    //// nodeを入れる
                    // currentに対応するnodeをnode_verticesに入れる
                    let nodes = current.create_node(vec![current_x, y_coord], *cnot_pos as i32);
                    let (node_id, node) = nodes;
                    node_vertices.insert(node_id.clone(), node.clone());
                    //// edgeを張る
                    // Noneでないものはpreviousがあればそれに対する辺を張る
                    if previous.is_some() {
                        if *cnot_pos == 0 {
                            // 前のはcnotではない
                            let previous = previous.clone().unwrap();
                            let previous_node_id = previous.get_node_id(if previous.is_control() {
                                let count = previous.control_count();
                                if count == 0 {
                                    Some(0)
                                } else {
                                    Some(previous.control_count() - 1)
                                }
                            } else {
                                None
                            });
                            undir_edges.insert(
                                format!("qedge_{}_node_{}", previous_node_id, node_id),
                                json::UndirEdgesValue {
                                    src: previous_node_id.clone(),
                                    tgt: node_id.clone(),
                                },
                            );
                        } else {
                            // println!("cnot_pos {:?}", *cnot_pos);
                            let previous_node_id = current.get_node_id(Some(*cnot_pos as i32 - 1));
                            // 前のはcnot
                            undir_edges.insert(
                                format!("qedge_cnot_{}_node_{}", cnot_pos, node_id),
                                json::UndirEdgesValue {
                                    src: previous_node_id.clone(),
                                    tgt: node_id.clone(),
                                },
                            );
                        }
                    }
                    // cnotの場合は対応するcontrol_fromがあるので、それに対する辺を張る
                    if current.is_controlled() {
                        let control_from_node_id = current.get_control_from_node_id();
                        undir_edges.insert(
                            format!("qedge_control_{}_node_{}", control_from_node_id, node_id),
                            json::UndirEdgesValue {
                                src: control_from_node_id.clone(),
                                tgt: node_id.clone(),
                            },
                        );
                    }
                }
            }
        })
    });
    // wire_verticesに最後のqubitを入れる
    qubit_cells.iter().enumerate().for_each(|(i, q)| {
        let y_coord = -(i as f64);
        wire_vertices.insert(
            q.output_qubit_id(),
            json::WireVerticesValue::create(false, vec![current_x + 1.0, y_coord]),
        );
        // 最後のnode_idを取り出し、最後のqubitに連結する辺を張る
        let last_op = q.get_last();
        if last_op.is_some() {
            let last_op = last_op.unwrap();
            let is_cnot = last_op.is_control();
            let last_node_id = last_op.get_node_id(if is_cnot {
                Some(last_op.control_count() - 1)
            } else {
                None
            });
            undir_edges.insert(
                format!("qedge_{}_last", q.qubit_id()),
                json::UndirEdgesValue {
                    src: last_node_id.clone(),
                    tgt: q.output_qubit_id().clone(),
                },
            );
        } else {
            undir_edges.insert(
                format!("qedge_{}_last", q.qubit_id()),
                json::UndirEdgesValue {
                    src: q.input_qubit_id().clone(),
                    tgt: q.output_qubit_id().clone(),
                },
            );
        }
    });
    json::PyzxCircuitJson {
        wire_vertices,
        node_vertices,
        undir_edges,
    }
}
#[cfg(test)]
mod tests {
    use crate::select_gates::simple_select_controls::*;
    use crate::{gates::toffoli, util::cellize};

    use super::*;
    #[test]
    fn test_serialize() {
        let mut wire_vertices = json::WireVertices::new();
        wire_vertices.insert(
            "b0".to_string(),
            json::WireVerticesValue {
                annotation: json::WireVerticesAnnotation {
                    boundary: true,
                    coord: vec![0.0, 0.0],
                    input: true,
                    output: false,
                },
            },
        );
        wire_vertices.insert(
            "b1".to_string(),
            json::WireVerticesValue {
                annotation: json::WireVerticesAnnotation {
                    boundary: true,
                    coord: vec![2.0, 0.0],
                    input: false,
                    output: true,
                },
            },
        );
        let mut node_vertices = json::NodeVertices::new();
        node_vertices.insert(
            "v0".to_string(),
            json::NodeVerticesValue {
                annotation: json::NodeVerticesAnnotation {
                    coord: vec![1.0, 0.0],
                },
                data: json::NodeVerticesData {
                    kind: "Z".to_string(),
                    value: Some("\\pi".to_string()),
                    is_edge: None,
                },
            },
        );
        let mut undir_edges = json::UndirEdges::new();
        undir_edges.insert(
            "e0".to_string(),
            json::UndirEdgesValue {
                src: "b0".to_string(),
                tgt: "v0".to_string(),
            },
        );
        undir_edges.insert(
            "e1".to_string(),
            json::UndirEdgesValue {
                src: "v0".to_string(),
                tgt: "b1".to_string(),
            },
        );
        let test_struct = json::PyzxCircuitJson {
            wire_vertices,
            node_vertices,
            undir_edges,
        };
        let serialized = serde_json::to_string(&test_struct).unwrap();
        println!("{}", serialized);
    }
    #[test]
    fn test_graph() {
        let qcell = cellize(primitive::Qubit::new("q1"));
        primitive::Qubit::gate(qcell.clone(), primitive::PrimitiveGate::H);
        let pyzx_json = to_pyzx_graph(vec![qcell.clone()]);
        println!("{:?}", pyzx_json);
    }
    #[test]
    fn test_circuit() {
        let q1 = cellize(primitive::Qubit::new("q1"));
        let q2 = cellize(primitive::Qubit::new("q2"));
        let q3 = cellize(primitive::Qubit::new("q3"));
        toffoli(q1.clone(), q2.clone(), q3.clone());
        let pyzx_json = to_pyzx_circuit(vec![q1, q2, q3]);
        println!("{:?}", pyzx_json);
    }
    #[test]
    fn test_incomplete() {
        let q1 = cellize(primitive::Qubit::new("q1"));
        let q2 = cellize(primitive::Qubit::new("q2"));
        primitive::Qubit::control(q2.clone());
        let from = primitive::Qubit::control(q1.clone());
        let target = primitive::Qubit::export(q2.clone());
        target.control_by(&from);
        let pyzx_json = to_pyzx_circuit(vec![q1, q2]);
        println!("{:?}", pyzx_json);
    }
    #[test]
    fn test_middle_complete() {
        let q1 = cellize(primitive::Qubit::new("q1"));
        let q2 = cellize(primitive::Qubit::new("q2"));
        let q3 = cellize(primitive::Qubit::new("q3"));
        let q4 = cellize(primitive::Qubit::new("q4"));
        let control = primitive::Qubit::control(q1.clone());
        let (leftc, _) = in_layer(&control, q2.clone(), q3.clone());
        // let leftc = in_layer(&control, q2.clone(), q3.clone());
        let export = primitive::Qubit::export(q4.clone());
        export.control_by(&leftc);
        // toffoli_first_control(&control, q2.clone(), q3.clone());
        let qcells = vec![q1.clone(), q2.clone(), q3.clone(), q4.clone()];
        println!("{:?}", qcells);
        let pyzx_json = to_pyzx_circuit(qcells);
        println!("{:?}", pyzx_json);
    }
}
