// QASM 2.0におけるオペレーションのファイル

// QASM
// 実質値なのでCopyもつける
#[derive(PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Copy, Clone)]
pub struct QubitId(pub i32);

// QASMの操作
// 実質値なのでCopyもつける
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum Operation {
    Z(QubitId),
    H(QubitId),
    X(QubitId),
    T(QubitId),
    TDag(QubitId),
    S(QubitId),
    SDag(QubitId),
    // 追加でCNOTがある
    CX(QubitId, QubitId),
}

impl ToString for Operation {
    fn to_string(&self) -> String {
        match &self {
            Operation::Z(target) => format!("z q[{}];", target.0),
            Operation::H(target) => format!("h q[{}];", target.0),
            Operation::X(target) => format!("x q[{}];", target.0),
            Operation::T(target) => format!("t q[{}];", target.0),
            Operation::TDag(target) => format!("tdg q[{}];", target.0),
            Operation::S(target) => format!("s q[{}];", target.0),
            Operation::SDag(target) => format!("sdg q[{}];", target.0),
            Operation::CX(from, to) => format!("cx q[{}], q[{}];", from.0, to.0),
        }
    }
}

#[derive(Debug, Clone)]
// QASM 2.0のファイルは基本的には操作の集合
pub struct File {
    pub qubit_count: usize,
    pub operations: Vec<Operation>,
}

impl ToString for File {
    fn to_string(&self) -> String {
        let header = "OPENQASM 2.0;\n";
        let includer = "include \"qelib1.inc\";\n";
        let qubit_declaration = format!("qreg q[{}];\n", self.qubit_count);
        let qasm_strings = self
            .operations
            .iter()
            .map(|operation| operation.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            "{}{}{}{}",
            header, includer, qubit_declaration, qasm_strings
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn always_pass() {
        let file = File {
            qubit_count: 2,
            operations: vec![
                Operation::Z(QubitId(0)),
                Operation::X(QubitId(1)),
                Operation::CX(QubitId(0), QubitId(1)),
            ],
        };
        let qasm_string = "OPENQASM 2.0;\n".to_string()
            + "include \"qelib1.inc\";\n"
            + "qreg q[2];\n"
            + "z q[0];\n"
            + "x q[1];\n"
            + "cx q[0], q[1];";
        assert_eq!(file.to_string(), qasm_string);
    }
}
