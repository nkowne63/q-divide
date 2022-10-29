use super::json;
use crate::primitive::{self, OperationCell};

pub trait QubitSerializeUtil {
    fn operation_count(&self) -> usize;
    fn raw_operation_count(&self) -> usize;
    fn qubit_id(&self) -> String;
    fn input_qubit_id(&self) -> String {
        format!("input_{}", self.qubit_id())
    }
    fn output_qubit_id(&self) -> String {
        format!("output_{}", self.qubit_id())
    }
    fn get_operation(&self, index: usize) -> Option<primitive::OperationCell>;
    fn iter(&self) -> QubitOperationsIter;
    fn get_first(&self) -> Option<primitive::OperationCell> {
        self.get_operation(0)
    }
    fn get_last(&self) -> Option<primitive::OperationCell> {
        let target = self.raw_operation_count() as i32 - 1;
        if target < 0 {
            return None;
        }
        self.get_operation(target as usize)
    }
}

impl QubitSerializeUtil for primitive::QubitCell {
    fn operation_count(&self) -> usize {
        self.as_ref()
            .borrow()
            .operations
            .iter()
            .map(|op| {
                let node = &op.as_ref().borrow().node_type;
                match node {
                    primitive::NodeType::ControlledNot(_, count) => count.clone() as usize,
                    _ => 1 as usize,
                }
            })
            .sum::<usize>()
    }
    fn raw_operation_count(&self) -> usize {
        self.as_ref().borrow().operations.len()
    }
    fn qubit_id(&self) -> String {
        self.as_ref().borrow().id.clone()
    }
    fn get_operation(&self, index: usize) -> Option<primitive::OperationCell> {
        self.as_ref()
            .borrow()
            .operations
            .get(index)
            .map(|c| c.clone())
    }
    fn iter(&self) -> QubitOperationsIter {
        QubitOperationsIter {
            index: 0,
            qcell: self.clone(),
            cnot_index: 0,
        }
    }
}

pub trait OperationSerializeUtil {
    // アクセサ系
    fn parent(&self) -> primitive::QubitCell;
    fn parent_id(&self) -> String;
    fn is_control(&self) -> bool;
    fn is_controlled(&self) -> bool;
    fn control_count(&self) -> i32;
    fn control_position(&self) -> i32;
    fn control_from_op(&self) -> Option<primitive::OperationCell>;
    // controlの時はposも考慮する
    fn get_node_id(&self, pos: Option<i32>) -> String;
    fn get_control_from_node_id(&self) -> String;
    // ノード生成系
    fn create_node(&self, start_coord: json::Coord, pos: i32) -> (String, json::NodeVerticesValue);
    fn create_nodes(&self, start_coord: json::Coord) -> Vec<(String, json::NodeVerticesValue)>;
}

impl OperationSerializeUtil for primitive::OperationCell {
    fn parent(&self) -> primitive::QubitCell {
        self.as_ref().borrow().parent.upgrade().unwrap().clone()
    }
    fn parent_id(&self) -> String {
        self.parent().qubit_id()
    }
    fn is_control(&self) -> bool {
        let node_type = &self.as_ref().borrow().node_type;
        if let primitive::NodeType::Control(_) = node_type {
            return true;
        } else {
            return false;
        }
    }
    fn is_controlled(&self) -> bool {
        let node_type = &self.as_ref().borrow().node_type;
        if let primitive::NodeType::ControlledNot(_, _) = node_type {
            return true;
        } else {
            return false;
        }
    }
    fn control_count(&self) -> i32 {
        let node_type = &self.as_ref().borrow().node_type;
        if let primitive::NodeType::Control(count) = node_type {
            return count.clone();
        } else {
            return 0;
        }
    }
    fn control_position(&self) -> i32 {
        let node_type = &self.as_ref().borrow().node_type;
        if let primitive::NodeType::ControlledNot(_, position) = node_type {
            return position.clone();
        } else {
            return 0;
        }
    }
    fn control_from_op(&self) -> Option<primitive::OperationCell> {
        let node_type = &self.as_ref().borrow().node_type;
        if let primitive::NodeType::ControlledNot(op, _) = node_type {
            let upgraded = op.upgrade().unwrap();
            return Some(upgraded);
        } else {
            return None;
        }
    }
    fn get_node_id(&self, pos: Option<i32>) -> String {
        let parent = self.parent_id();
        let operation_id = self.as_ref().borrow().id.clone();
        if let Some(pos) = pos {
            return format!("qnode_{}_op{}_pos{}", parent, operation_id, pos);
        } else {
            return format!("qnode_{}_op{}", parent, operation_id);
        }
    }
    fn get_control_from_node_id(&self) -> String {
        let control_from_op = self.control_from_op().unwrap();
        let control_position = self.control_position();
        control_from_op.get_node_id(Some(control_position))
    }
    fn create_node(&self, coord: json::Coord, pos: i32) -> (String, json::NodeVerticesValue) {
        if self.is_control() {
            let node_id = self.get_node_id(Some(pos));
            let node_value = json::NodeVerticesValue {
                annotation: json::NodeVerticesAnnotation {
                    coord: vec![coord[0], coord[1]],
                },
                data: json::NodeVerticesData {
                    kind: "Z".to_string(),
                    value: None,
                    is_edge: None,
                },
            };
            (node_id, node_value)
        } else {
            let node_id = self.get_node_id(None);
            let node_annotation = json::NodeVerticesAnnotation {
                coord: coord.clone(),
            };
            let node_type = &self.as_ref().borrow().node_type;
            let node_value = match node_type {
                primitive::NodeType::Control(_) => panic!("unreachable condition"),
                primitive::NodeType::ControlledNot(_, _) => json::NodeVerticesData {
                    kind: "X".to_string(),
                    value: None,
                    is_edge: None,
                },
                primitive::NodeType::PreControlledNot => json::NodeVerticesData {
                    kind: "X".to_string(),
                    value: None,
                    is_edge: None,
                },
                primitive::NodeType::PrimitiveGate(gate) => match gate {
                    primitive::PrimitiveGate::Z => json::NodeVerticesData {
                        kind: "Z".to_string(),
                        value: Some("\\pi".to_string()),
                        is_edge: None,
                    },
                    primitive::PrimitiveGate::X => json::NodeVerticesData {
                        kind: "X".to_string(),
                        value: Some("\\pi".to_string()),
                        is_edge: None,
                    },
                    primitive::PrimitiveGate::T => json::NodeVerticesData {
                        kind: "Z".to_string(),
                        value: Some("\\pi/4".to_string()),
                        is_edge: None,
                    },
                    primitive::PrimitiveGate::S => json::NodeVerticesData {
                        kind: "Z".to_string(),
                        value: Some("\\pi/2".to_string()),
                        is_edge: None,
                    },
                    primitive::PrimitiveGate::TDag => json::NodeVerticesData {
                        kind: "Z".to_string(),
                        value: Some("-\\pi/4".to_string()),
                        is_edge: None,
                    },
                    primitive::PrimitiveGate::SDag => json::NodeVerticesData {
                        kind: "Z".to_string(),
                        value: Some("-\\pi/2".to_string()),
                        is_edge: None,
                    },
                    primitive::PrimitiveGate::H => json::NodeVerticesData {
                        kind: "hadamard".to_string(),
                        value: None,
                        is_edge: Some("true".to_string()),
                    },
                },
            };
            // 場合分けしていく
            return (
                node_id,
                json::NodeVerticesValue {
                    annotation: node_annotation,
                    data: node_value,
                },
            );
        }
    }
    fn create_nodes(&self, start_coord: json::Coord) -> Vec<(String, json::NodeVerticesValue)> {
        if self.is_control() {
            let count = self.control_count();
            (0..count)
                .map(|pos| {
                    let coord = vec![start_coord[0] + pos as f64, start_coord[1]];
                    self.create_node(coord, pos)
                })
                .collect::<Vec<_>>()
        } else {
            return vec![self.create_node(start_coord, 0)];
        }
    }
}

#[derive(Clone, Debug)]
pub struct QubitOperationsIter {
    qcell: primitive::QubitCell,
    // nextに使うためのindex
    index: usize,
    cnot_index: usize,
}
impl QubitOperationsIter {
    pub fn get_item(&self) -> Option<QubitOperationsIterItem> {
        if self.index >= self.qcell.raw_operation_count() {
            return None;
        }
        // 範囲内の場合は返すものを組み立てる
        let current = self.qcell.get_operation(self.index).unwrap();
        let previous = if self.index == 0 && (!current.is_control() || self.cnot_index == 0) {
            // 一番最初のノードは前のノードがない
            None
        } else if current.is_control() && self.cnot_index == 0 {
            // cnotの最初だった場合は前のを返す
            self.qcell.get_operation(self.index - 1)
        } else if current.is_control() && self.cnot_index > 0 {
            // 前のもcnot
            self.qcell.get_operation(self.index)
        } else {
            // cnotじゃない場合は前のを返す
            self.qcell.get_operation(self.index - 1)
        };

        let is_current_control = current.is_control();

        return Some(QubitOperationsIterItem {
            previous,
            current,
            cnot_pos: if is_current_control {
                self.cnot_index
            } else {
                0
            },
        });
    }
}
#[derive(Clone, Debug)]
pub struct QubitOperationsIterItem {
    pub previous: Option<primitive::OperationCell>,
    pub current: primitive::OperationCell,
    pub cnot_pos: usize,
}
impl Iterator for QubitOperationsIter {
    type Item = QubitOperationsIterItem;
    // cnotの場合、そのcnotの何番目かは返されないが、positionでそれは頑張る
    fn next(&mut self) -> Option<Self::Item> {
        let item = self.get_item();
        if item.is_none() {
            return None;
        };
        let QubitOperationsIterItem {
            previous: _,
            current,
            cnot_pos: _,
        } = item.clone().unwrap();
        // println!(
        //     "current {:?} {:?}",
        //     current.control_count(),
        //     self.cnot_index
        // );

        // 状態更新
        self.index += if current.is_control() {
            if (self.cnot_index as i32) < current.control_count() - 1 {
                // cnotを進んでる間はindexは変化しない
                // println!("here?");
                0
            } else {
                1
            }
        } else {
            1
        };
        self.cnot_index = if current.is_control() {
            self.cnot_index + 1
        } else {
            0
        };

        return item;
    }
}

#[derive(Clone, Debug)]
pub struct QubitsIter {
    qiters: Vec<QubitOperationsIter>,
}
#[derive(Clone, Debug)]
pub struct QubitsIterItem {
    pub items: Vec<Option<<QubitOperationsIter as Iterator>::Item>>,
}
impl QubitsIter {
    pub fn new(qcells: Vec<primitive::QubitCell>) -> Self {
        let qiters = qcells.iter().map(|qcell| qcell.clone().iter()).collect();
        QubitsIter { qiters }
    }
}
impl Iterator for QubitsIter {
    type Item = QubitsIterItem;
    #[allow(clippy::if_same_then_else)]
    fn next(&mut self) -> Option<Self::Item> {
        // currentのものからまず出力できるものを作る
        let current_items = self
            .qiters
            .iter()
            .map(|qiter| qiter.get_item())
            .collect::<Vec<_>>();
        let get_cnot_infos = |iter: QubitsIter| {
            let current_list = iter
                .qiters
                .iter()
                .map(|qiter| qiter.get_item())
                .map(|item_opt| item_opt.map(|item| item.current))
                .collect::<Vec<_>>();
            let cnot_indexes = current_list
                .iter()
                .enumerate()
                .filter_map(|(idx, op)| {
                    op.clone().as_ref().map_or(None, |op_cell| {
                        if op_cell.is_control() || op_cell.is_controlled() {
                            Some(idx)
                        } else {
                            None
                        }
                    })
                })
                .collect::<Vec<_>>();
            let cnot_has_correspondense_indexes = cnot_indexes
                .iter()
                .filter_map(|index| {
                    let get_cnot_pos = |index: &usize| {
                        let cnot_op = current_list[*index].as_ref().unwrap().clone();

                        let cnot_pos = if cnot_op.is_controlled() {
                            cnot_op.control_position()
                        } else {
                            iter.qiters[*index].cnot_index as i32
                        };
                        return (cnot_op, cnot_pos);
                    };
                    fn control_from_qubit_id(opc: OperationCell) -> Option<String> {
                        if opc.is_control() {
                            return Some(opc.parent_id());
                        } else if opc.is_controlled() {
                            return Some(opc.control_from_op().unwrap().parent_id());
                        } else {
                            return None;
                        }
                    }

                    let (cnot_op, cnot_pos) = get_cnot_pos(index);

                    let corresponsense_index = cnot_indexes.iter().find(|&idx| {
                        if idx == index {
                            false
                        } else {
                            let (temp_cnot_op, temp_cnot_pos) = get_cnot_pos(idx);
                            if temp_cnot_pos == cnot_pos {
                                let temp_from_id = control_from_qubit_id(temp_cnot_op);
                                let from_id = control_from_qubit_id(cnot_op.clone());
                                temp_from_id == from_id
                            } else {
                                false
                            }
                        }
                    });

                    if corresponsense_index.is_none() {
                        return None;
                    }

                    return Some(*index);
                })
                .collect::<Vec<_>>();
            return (cnot_indexes, cnot_has_correspondense_indexes);
        };
        // currentのリストを取得して、cnotのペアが揃ってないやつ以外を除外する
        let (cnot_indexes, cnot_has_correspondense_indexes) = get_cnot_infos(self.clone());
        // println!(
        //     "cnot {:?} {:?}",
        //     current_items, cnot_has_correspondense_indexes
        // );
        // let dur = std::time::Duration::from_millis(1000);
        // std::thread::sleep(dur);
        let return_items = current_items
            .clone()
            .into_iter()
            .enumerate()
            .map(|(index, item)| {
                if !cnot_indexes.contains(&index) {
                    return item;
                } else if cnot_has_correspondense_indexes.contains(&index) {
                    return item;
                } else {
                    let op = current_items[index].as_ref().unwrap().clone().current;
                    let control_count = op.control_count();
                    if control_count == 0 {
                        return item;
                    }
                    return None;
                }
            })
            .collect::<Vec<_>>();
        // println!("{:?}", return_items);
        // cnotのペアが揃っていないもの以外をnextする
        // ただし、control_countがzeroのものはnextする
        self.qiters
            .iter_mut()
            .enumerate()
            .for_each(|(index, qiter)| {
                if !cnot_indexes.contains(&index) {
                    qiter.next();
                } else if cnot_has_correspondense_indexes.contains(&index) {
                    qiter.next();
                } else {
                    let op = current_items[index].as_ref().unwrap().clone().current;
                    let control_count = op.control_count();
                    // println!("{:?} {:?}", op.control_count(), op.parent_id());
                    if control_count == 0 && op.is_control() {
                        // println!("reached");
                        qiter.next();
                    }
                }
            });

        // 値を返却する
        if current_items.iter().all(|item| item.is_none()) {
            return None;
        }
        return Some(QubitsIterItem {
            items: return_items,
        });
    }
}
