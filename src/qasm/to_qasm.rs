use super::operations;
use super::serialize_utils;
use crate::primitive;

pub fn to_qasm(qubit_cells: Vec<primitive::QubitCell>) -> operations::File {
    let qubit_count = qubit_cells.len();
    let qubit_id_map = serialize_utils::QubitIdMap::from_cells(&qubit_cells);
    let mut next_operations = serialize_utils::NextOperations::initialize_from_cells(&qubit_cells);
    let mut qasm_operations = Vec::new();
    let mut count = 0;
    while next_operations.has_next() {
        // assert!(count < 1000, "Too many iterations");
        let operation = next_operations.to_qasm(&qubit_id_map);
        if operation.is_empty() {
            break;
        }
        qasm_operations.extend(operation);
        next_operations.next();
        count += 1;
    }
    operations::File {
        qubit_count,
        operations: qasm_operations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gates::toffoli;
    use crate::primitive::Qubit;
    use crate::util::cellize;
    #[test]
    fn toffoli_test() {
        let q1 = cellize(Qubit::new("q1"));
        let q2 = cellize(Qubit::new("q2"));
        let q3 = cellize(Qubit::new("q3"));
        toffoli(q1.clone(), q2.clone(), q3.clone());
        let qasm_file = to_qasm(vec![q1.clone(), q2.clone(), q3.clone()]);
        println!("{}", qasm_file.to_string());
    }
    #[test]
    fn uniform_layered_test() {
        use crate::gates::*;
        let n = 4;
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

        let mut qubits = Vec::new();
        qubits.extend(datas.clone());
        qubits.extend(ancillas.clone());
        qubits.extend(targets.clone());
        qubits.push(first_qubit.clone());

        // println!("{:#?}", qubits);

        let qasm_file = to_qasm(qubits);
        println!("{}", qasm_file.to_string());
    }
}
