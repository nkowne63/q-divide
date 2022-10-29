use crate::util::*;
use std::cell::RefCell;
use std::hash::{Hash, Hasher};
use std::rc::{Rc, Weak};

#[derive(Debug)]
pub enum PrimitiveGate {
    Z,
    H,
    X,
    T,
    TDag,
    S,
    SDag,
}

#[derive(Debug)]
pub struct Operation {
    pub id: usize,
    pub parent: Weak<RefCell<Qubit>>,
    pub node_type: NodeType,
}
pub type OperationCell = Rc<RefCell<Operation>>;
impl PartialEq for Operation {
    fn eq(&self, other: &Self) -> bool {
        let parent_self = self.parent.upgrade();
        let parent_other = other.parent.upgrade();
        if let (Some(qcell_self), Some(qcell_other)) = (parent_self, parent_other) {
            self.id == other.id && qcell_self == qcell_other
        } else {
            // parentが存在しない場合はfalse
            return false;
        }
    }
}
impl Eq for Operation {}
impl Hash for Operation {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[derive(Debug)]
pub enum NodeType {
    PrimitiveGate(PrimitiveGate),
    PreControlledNot,
    // control total count
    Control(i32),
    // control from position
    ControlledNot(Weak<RefCell<Operation>>, i32),
}

#[derive(Debug, Clone)]
pub struct ControlFrom {
    pub operation: OperationCell,
}

#[derive(Debug)]
pub struct ControlTarget {
    pub operation: OperationCell,
}

#[derive(Debug)]
pub struct Qubit {
    pub id: String,
    pub operations: Vec<OperationCell>,
}
pub type QubitCell = Rc<RefCell<Qubit>>;
impl PartialEq for Qubit {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for Qubit {}
impl Hash for Qubit {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Qubit {
    pub fn new(id: &str) -> Qubit {
        Qubit {
            id: id.to_string(),
            operations: Vec::new(),
        }
    }
    pub fn control(qcell: QubitCell) -> ControlFrom {
        let length = qcell.borrow().operations.len();
        let new_operation = cellize(Operation {
            id: length + 1,
            parent: Rc::downgrade(&qcell),
            node_type: NodeType::Control(0),
        });
        qcell.borrow_mut().operations.push(new_operation.clone());
        let control_from = ControlFrom {
            operation: new_operation.clone(),
        };
        return control_from;
    }
    pub fn export(qcell: QubitCell) -> ControlTarget {
        let length = qcell.borrow().operations.len();
        let new_operation = cellize(Operation {
            id: length + 1,
            parent: Rc::downgrade(&qcell),
            node_type: NodeType::PreControlledNot,
        });
        qcell.borrow_mut().operations.push(new_operation.clone());
        let control_target = ControlTarget {
            operation: new_operation.clone(),
        };
        return control_target;
    }
    pub fn gate(qcell: QubitCell, gate: PrimitiveGate) {
        let length = qcell.borrow().operations.len();
        let new_operation = cellize(Operation {
            id: length + 1,
            parent: Rc::downgrade(&qcell),
            node_type: NodeType::PrimitiveGate(gate),
        });
        qcell.borrow_mut().operations.push(new_operation.clone());
    }
}

impl ControlTarget {
    pub fn control_by(self, control_from: &ControlFrom) {
        let control_from_operation = control_from.operation.clone();
        let mut control_from_operation = control_from_operation.borrow_mut();
        if let NodeType::Control(count) = control_from_operation.node_type {
            control_from_operation.node_type = NodeType::Control(count + 1);
            let operation = self.operation.clone();
            let mut operation = operation.borrow_mut();
            operation.node_type =
                NodeType::ControlledNot(Rc::downgrade(&control_from.operation), count);
        } else {
            panic!("control_from_operation is not control");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn control_not() {
        let qcell1 = cellize(Qubit::new("1"));
        let qcell2 = cellize(Qubit::new("2"));
        let control_from = Qubit::control(qcell1.clone());
        let control_target = Qubit::export(qcell2.clone());
        control_target.control_by(&control_from);
        println!("qcell1, {:?}", qcell1);
        println!("qcell2, {:?}", qcell2);
        let control_target_cell = qcell2.clone();
        let control_target_operation = &control_target_cell.borrow().operations[0];
        let control_target_operation = control_target_operation.clone();
        let control_target_node_type = &control_target_operation.borrow().node_type;
        if let NodeType::ControlledNot(control_from, _) = control_target_node_type {
            let control_from_operation = control_from.upgrade().unwrap();
            let control_from_cell = control_from_operation.borrow().parent.upgrade().unwrap();
            assert_eq!(control_from_cell, qcell1);
        }
    }
    #[test]
    fn double_control() {
        let qcell1 = cellize(Qubit::new("1"));
        let qcell2 = cellize(Qubit::new("2"));
        let control_from = Qubit::control(qcell1.clone());
        let control_target1 = Qubit::export(qcell2.clone());
        let control_target2 = Qubit::export(qcell2.clone());
        control_target1.control_by(&control_from);
        control_target2.control_by(&control_from);
        println!("qcell1, {:?}", qcell1);
        println!("qcell2, {:?}", qcell2);
        let control_target1_cell = qcell2.clone();
        let operation_len = control_target1_cell.borrow().operations.len();
        assert_eq!(operation_len, 2);
    }
    #[test]
    fn control_count() {
        let qcell1 = cellize(Qubit::new("1"));
        let qcell2 = cellize(Qubit::new("2"));
        let control_from = Qubit::control(qcell1.clone());
        let control_target1 = Qubit::export(qcell2.clone());
        let control_target2 = Qubit::export(qcell2.clone());
        control_target1.control_by(&control_from);
        control_target2.control_by(&control_from);
        let control_from_cell = qcell1.clone();
        let control_from_operation = &control_from_cell.borrow().operations[0];
        let control_from_operation_node_type = &control_from_operation.borrow().node_type;
        let count = match control_from_operation_node_type {
            &NodeType::Control(count) => Some(count),
            _ => None,
        };
        assert_eq!(count, Some(2));
    }
}
