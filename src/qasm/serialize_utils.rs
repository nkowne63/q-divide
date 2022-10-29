use super::operations;
use crate::primitive;
use itertools::Itertools;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

type PrimitiveQubitId = String;

pub struct QubitIdMap(HashMap<PrimitiveQubitId, operations::QubitId>);

impl QubitIdMap {
    pub fn from_cells(qubit_cells: &Vec<primitive::QubitCell>) -> QubitIdMap {
        let mut qubit_id_map = HashMap::new();
        qubit_cells
            .iter()
            .enumerate()
            .for_each(|(qubit_index, cell)| {
                let qubit_name: PrimitiveQubitId = cell.borrow().id.clone();
                let qubit_id = operations::QubitId(qubit_index as i32);
                qubit_id_map.insert(qubit_name, qubit_id);
            });
        QubitIdMap(qubit_id_map)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NextOperation {
    pub qubit_cell: primitive::QubitCell,
    pub operation_index: usize,
    // ControlFromだった場合、次が何番目なのか
    pub cnot_count: i32,
}

impl NextOperation {
    fn new(qubit_cell: &primitive::QubitCell) -> Option<Self> {
        if qubit_cell.clone().borrow().operations.len() == 0 {
            return None;
        }
        Some(Self {
            qubit_cell: qubit_cell.clone(),
            operation_index: 0,
            cnot_count: 0,
        })
    }
    fn has_next(&self) -> bool {
        let qubit_cell = self.qubit_cell.clone();
        let operation_max_count = qubit_cell.borrow().operations.len();
        let last_cnot_count = match qubit_cell.borrow().operations[operation_max_count - 1]
            .clone()
            .borrow()
            .node_type
        {
            primitive::NodeType::Control(count) => count,
            _ => 0,
        };
        (self.operation_index + 1 < operation_max_count) || (self.cnot_count + 1 < last_cnot_count)
    }
    fn next(&mut self) {
        if !self.has_next() {
            return;
        }
        if self.is_control() {
            // controlだった場合、カウント最大かどうかが判定の分かれ目になる
            let qubit_cell = self.qubit_cell.clone();
            let operation = qubit_cell.borrow().operations[self.operation_index].clone();
            let node_type = &operation.borrow().node_type;
            let max_cnot_count = match node_type {
                primitive::NodeType::Control(count) => *count,
                // primitive::NodeType::ControlledNotの可能性もある
                _ => 0,
            };
            if !(self.cnot_count + 1 < max_cnot_count) {
                // カウント最大の場合、cnot_countをリセットして次に進む
                self.cnot_count = 0;
                self.operation_index += 1;
            } else {
                // カウント最大ではない場合、cnot_countを上げる
                self.cnot_count += 1;
            }
        } else {
            // 非controlだった場合、カウントを0にして次に進む
            self.cnot_count = 0;
            self.operation_index += 1;
        }
        // 次がcnotのfromでcountが0の場合、さらに次に進む
        let current_operation_index = self.operation_index;
        let qubit_cell = self.qubit_cell.clone();
        let current_operation = &qubit_cell.borrow().operations[current_operation_index];
        let current_operation_node_type = &current_operation.borrow().node_type;
        if let primitive::NodeType::Control(count) = current_operation_node_type {
            if *count == 0 {
                self.next();
            }
        }
    }
    fn is_control(&self) -> bool {
        let qubit_cell = self.qubit_cell.clone();
        let operation = qubit_cell.borrow().operations[self.operation_index].clone();
        let node_type = &operation.borrow().node_type;
        match node_type {
            primitive::NodeType::Control(_) => true,
            primitive::NodeType::ControlledNot(_, _) => true,
            _ => false,
        }
    }
    fn extract_parent_operation(&self) -> Option<(Rc<RefCell<primitive::Operation>>, i32)> {
        let qubit_cell = self.qubit_cell.clone();
        let operation = qubit_cell.borrow().operations[self.operation_index].clone();
        let node_type = &operation.borrow().node_type;
        let parent_info = match node_type {
            primitive::NodeType::ControlledNot(weak, position) => {
                Some((weak.upgrade().unwrap(), *position))
            }
            _ => None,
        };
        parent_info
    }
    fn is_control_pair(na: &Self, nb: &Self) -> bool {
        // どちらかがcontrolでなければ違う
        if !(na.is_control() && nb.is_control()) {
            return false;
        }
        let operation_na = na.qubit_cell.clone().borrow().operations[na.operation_index].clone();
        let operation_nb = nb.qubit_cell.clone().borrow().operations[nb.operation_index].clone();
        let parent_operation_na = na.extract_parent_operation();
        let parent_operation_nb = nb.extract_parent_operation();

        // どちらかがControlledNotである必要がある
        match (parent_operation_na, parent_operation_nb) {
            // nbが親の可能性がある
            (Some(parent_operation_na), None) => {
                return parent_operation_na.0 == operation_nb
                    && nb.cnot_count == parent_operation_na.1
            }
            // naが親の可能性がある
            (None, Some(parent_operation_nb)) => {
                return parent_operation_nb.0 == operation_na
                    && na.cnot_count == parent_operation_nb.1
            }
            _ => return false,
        }
    }
    fn to_qasm(&self, qubit_id_map: &QubitIdMap) -> Option<operations::Operation> {
        // Controlの場合はnone
        if self.is_control() {
            return None;
        }
        let qubit_cell = self.qubit_cell.clone();
        let operation = qubit_cell.borrow().operations[self.operation_index].clone();
        let node_type = &operation.borrow().node_type;
        let gate = match node_type {
            primitive::NodeType::PrimitiveGate(gate) => gate,
            _ => return None,
        };
        let qubit_id_string = &qubit_cell.borrow().id;
        let qubit_id_i32 = qubit_id_map.0.get(qubit_id_string).unwrap();
        // あとは変換するだけ
        Some(match gate {
            primitive::PrimitiveGate::Z => operations::Operation::Z(*qubit_id_i32),
            primitive::PrimitiveGate::H => operations::Operation::H(*qubit_id_i32),
            primitive::PrimitiveGate::X => operations::Operation::X(*qubit_id_i32),
            primitive::PrimitiveGate::T => operations::Operation::T(*qubit_id_i32),
            primitive::PrimitiveGate::TDag => operations::Operation::TDag(*qubit_id_i32),
            primitive::PrimitiveGate::S => operations::Operation::S(*qubit_id_i32),
            primitive::PrimitiveGate::SDag => operations::Operation::SDag(*qubit_id_i32),
        })
    }
    fn to_qasm_pair(
        na: &Self,
        nb: &Self,
        qubit_id_map: &QubitIdMap,
    ) -> Option<operations::Operation> {
        // control_pairである必要がある
        if !Self::is_control_pair(na, nb) {
            return None;
        }
        let qubit_cell_na = na.qubit_cell.clone();
        let qubit_cell_nb = nb.qubit_cell.clone();
        let parent_operation_na = na.extract_parent_operation();
        let parent_operation_nb = nb.extract_parent_operation();
        let qubit_id_string_na = &qubit_cell_na.borrow().id;
        let qubit_id_string_nb = &qubit_cell_nb.borrow().id;
        let qubit_id_i32_na = qubit_id_map.0.get(qubit_id_string_na).unwrap();
        let qubit_id_i32_nb = qubit_id_map.0.get(qubit_id_string_nb).unwrap();

        match (parent_operation_na, parent_operation_nb) {
            // naが親の場合
            (None, Some(_)) => {
                return Some(operations::Operation::CX(
                    *qubit_id_i32_na,
                    *qubit_id_i32_nb,
                ));
            }
            // nbが親の場合
            (Some(_), None) => {
                return Some(operations::Operation::CX(
                    *qubit_id_i32_nb,
                    *qubit_id_i32_na,
                ));
            }
            _ => return None,
        }
    }
}

// Noneだった場合は次は存在しない
#[derive(Debug, Clone)]
pub struct NextOperations(Vec<Option<NextOperation>>);

impl NextOperations {
    pub fn initialize_from_cells(qubit_cells: &Vec<primitive::QubitCell>) -> Self {
        Self(
            qubit_cells
                .iter()
                .map(|cell| NextOperation::new(&cell))
                .collect::<Vec<_>>(),
        )
    }
    pub fn next(&mut self) {
        // extractableなものをすべて進める
        let forwardable_index = self.pick_extractable_indexes().0;
        let operations = &mut self.0;
        forwardable_index.iter().for_each(|index| {
            let operation = &mut operations[*index];
            if let Some(operation) = operation {
                // Someの場合は2通りある
                if operation.has_next() {
                    operation.next();
                } else {
                    operations[*index] = None;
                }
            } else {
                // Noneの場合は次には進められない
                return;
            }
        });
    }
    pub fn has_next(&self) -> bool {
        // すべてがNoneのときだけfalseになる
        !self.0.iter().all(|n| n.is_none())
    }
    pub fn to_qasm(&self, qubit_id_map: &QubitIdMap) -> Vec<operations::Operation> {
        let (_, single_gate, cx_gate) = self.pick_extractable_indexes();
        let operations = &self.0;
        let mut qasm_vec = Vec::new();
        let single_qasm = single_gate
            .iter()
            .filter_map(|index| {
                let operation = &operations[*index];
                operation.as_ref().unwrap().to_qasm(qubit_id_map)
            })
            .collect::<Vec<_>>();
        let cx_qasm = cx_gate
            .iter()
            .filter_map(|index| {
                let operation_1 = &operations[index.0];
                let operation_2 = &operations[index.1];
                NextOperation::to_qasm_pair(
                    &operation_1.as_ref().unwrap(),
                    &operation_2.as_ref().unwrap(),
                    qubit_id_map,
                )
            })
            .collect::<Vec<_>>();
        qasm_vec.extend(single_qasm);
        qasm_vec.extend(cx_qasm);
        qasm_vec
    }
    // すべてのindex, 1qubit-gate, 2qubit-gateのペア
    fn pick_extractable_indexes(&self) -> (Vec<usize>, Vec<usize>, Vec<(usize, usize)>) {
        let operations = &self.0;
        let operation_is_control = operations
            .iter()
            .enumerate()
            .filter_map(|(index, operation)| {
                if let Some(operation) = operation {
                    return Some((index, operation.is_control()));
                } else {
                    return None;
                }
            })
            .collect::<Vec<_>>();
        let single_operations_index = operation_is_control
            .iter()
            .filter_map(|(index, is_control)| if !*is_control { Some(*index) } else { None })
            .collect::<Vec<_>>();
        let raw_cx_operations_index = operation_is_control
            .iter()
            .filter_map(|(index, is_control)| if *is_control { Some(*index) } else { None })
            .collect::<Vec<_>>();
        // ペアになってるものだけ抜き出す
        let cx_operations_index = raw_cx_operations_index
            .iter()
            .cartesian_product(raw_cx_operations_index.iter())
            .filter_map(|(index_1, index_2)| {
                if index_1 >= index_2 {
                    return None;
                }
                let operation_1 = &operations[*index_1].as_ref().unwrap();
                let operation_2 = &operations[*index_2].as_ref().unwrap();
                let is_pair = NextOperation::is_control_pair(*operation_1, *operation_2);
                if !is_pair {
                    return None;
                }
                Some((*index_1, *index_2))
            })
            .collect::<Vec<_>>();
        let all_operations_index = single_operations_index
            .iter()
            .chain(
                cx_operations_index
                    .iter()
                    .flat_map(|(index_1, index_2)| vec![index_1, index_2].into_iter()),
            )
            .map(|index| *index)
            .collect::<Vec<_>>();
        (
            all_operations_index,
            single_operations_index,
            cx_operations_index,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        gates::toffoli,
        primitive::{PrimitiveGate, Qubit, QubitCell},
        util::cellize,
    };
    fn get_qubit_cells() -> Vec<QubitCell> {
        let q1 = cellize(Qubit::new("test_1"));
        let q2 = cellize(Qubit::new("test_2"));
        Qubit::gate(q1.clone(), PrimitiveGate::H);
        Qubit::gate(q1.clone(), PrimitiveGate::T);
        Qubit::gate(q2.clone(), PrimitiveGate::Z);
        crate::gates::cnot(q1.clone(), q2.clone());
        Qubit::gate(q2.clone(), PrimitiveGate::T);
        Qubit::gate(q2.clone(), PrimitiveGate::H);
        Qubit::gate(q1.clone(), PrimitiveGate::Z);
        vec![q1.clone(), q2.clone()]
    }
    #[test]
    fn next_operation_new() {
        let qc1 = get_qubit_cells()[0].clone();
        let expectation = Some(NextOperation {
            qubit_cell: qc1.clone(),
            operation_index: 0,
            cnot_count: 0,
        });
        assert_eq!(NextOperation::new(&qc1), expectation);
    }
    #[test]
    fn next_operation_has_next() {
        let qc1 = get_qubit_cells()[0].clone();
        let nqc1 = NextOperation {
            qubit_cell: qc1.clone(),
            operation_index: 3,
            cnot_count: 0,
        };
        assert_eq!(nqc1.has_next(), false);
    }
    #[test]
    fn next_operation_has_next_2() {
        let qc1 = get_qubit_cells()[0].clone();
        let qc2 = get_qubit_cells()[1].clone();
        let qc1c = Qubit::control(qc1.clone());
        let qc2e1 = Qubit::export(qc2.clone());
        let qc2e2 = Qubit::export(qc2.clone());
        let qc2e3 = Qubit::export(qc2.clone());
        qc2e1.control_by(&qc1c);
        qc2e2.control_by(&qc1c);
        qc2e3.control_by(&qc1c);
        let nqc1 = NextOperation {
            qubit_cell: qc1.clone(),
            operation_index: 4,
            cnot_count: 2,
        };
        assert_eq!(nqc1.has_next(), false);
    }

    #[test]
    fn toffoli_test() {
        let q1 = cellize(Qubit::new("q1"));
        let q2 = cellize(Qubit::new("q2"));
        let q3 = cellize(Qubit::new("q3"));
        toffoli(q1.clone(), q2.clone(), q3.clone());
        let qcells = vec![q1.clone(), q2.clone(), q3.clone()];
        let map = QubitIdMap::from_cells(&qcells);
        let mut nops = NextOperations::initialize_from_cells(&qcells);
        nops.next();
        nops.next();
        nops.next();
        nops.next();
        nops.next();
        nops.next();
        nops.next();
        nops.next();
        nops.next();
        println!("10th {:#?}", nops.to_qasm(&map));
        nops.next();
        println!("11th {:#?}", nops.to_qasm(&map));
        assert_eq!(nops.has_next(), false);
    }
    #[test]
    fn uniform_layered_test() {
        use crate::gates::*;
        let n = 1;
        let first_qubit = cellize(Qubit::new("first"));
        let first_control = Qubit::control(first_qubit.clone());
        let datas = (0..n)
            .map(|i| cellize(Qubit::new(format!("data_{}", i).as_str())))
            .collect::<Vec<_>>();
        let ancillas = (0..n)
            .map(|i| cellize(Qubit::new(format!("ancilla_{}", i).as_str())))
            .collect::<Vec<_>>();
        let controls = in_over_2n(n, &first_control, datas.clone(), ancillas.clone());
        let data_length = controls.len();
        let targets = (0..n)
            .map(|i| cellize(Qubit::new(format!("target_{}", i).as_str())))
            .collect::<Vec<_>>();
        let random_data = generate_random_datas(data_length, 1);
        inject_qrom_datas(targets.clone(), controls, random_data);

        let mut qubit_cells = Vec::new();
        qubit_cells.extend(datas.clone());
        qubit_cells.extend(ancillas.clone());
        qubit_cells.extend(targets.clone());
        qubit_cells.push(first_qubit.clone());

        // println!("{:#?}", qubits);
        let _map = QubitIdMap::from_cells(&qubit_cells);
        let mut nop = NextOperations::initialize_from_cells(&qubit_cells);
        // dbg!(nop.to_qasm(&_map));

        nop.next();

        nop.next();
        nop.next();
        nop.next();
        nop.next();
        nop.next();
        nop.next();
        nop.next();
        nop.next();
        nop.next();
        // dbg!(nop.to_qasm(&_map));
        nop.next();
        // dbg!(nop.to_qasm(&_map));
        // dbg!(nop
        //     .0
        //     .iter()
        //     .enumerate()
        //     .map(|(idx, opt)| (idx, opt.is_some()))
        //     .collect::<Vec<_>>());
        // dbg!(&nop.0[0]);
        // dbg!(&nop
        //     .0
        //     .iter()
        //     .enumerate()
        //     .map(|(idx, op)| (idx, op.as_ref().unwrap().is_control()))
        //     .collect::<Vec<_>>());
        // dbg!(&nop);
    }
}
