from __future__ import annotations
from qiskit import QuantumCircuit, assemble, QuantumRegister, ClassicalRegister
import numpy as np
import sys

def _parse_qobj_dict(qasm: str) -> dict:
    """Parse qasm to qobj_dict

    Args:
        qasm (str): qasm string

    Returns:
        dict: qobj dict
    """
    qiskit_circuit = QuantumCircuit.from_qasm_str(qasm)
    qobj = assemble(qiskit_circuit)
    qobj_dict = qobj.to_dict()
    return qobj_dict

# cyclomatic complexity 1 + 4 = 5
def add_toffoli(circuit: QuantumCircuit, t1: int, t2: int, t3: int, c1: int, c2: int) -> None:
    # print(t1,t2,t3)
    if c1 == 1:
        circuit.x(t1)
    if c2 == 1:
        circuit.x(t2)
    circuit.ccx(t1,t2,t3)
    if c1 == 1:
        circuit.x(t1)
    if c2 == 1:
        circuit.x(t2)

# cyclomatic complexity 1 + 4 = 5
def add_start_toffoli(circuit: QuantumCircuit, t1: int, t2: int, t3: int, c1: int, c2: int) -> None:
    if c1 == 1:
        circuit.x(t1)
    if c2 == 1:
        circuit.x(t2)
    circuit.reset(t3)
    circuit.h(t3)
    circuit.t(t3)
    circuit.cx(t2, t3)
    circuit.tdg(t3)
    circuit.cx(t1, t3)
    circuit.t(t3)
    circuit.cx(t2, t3)
    circuit.tdg(t3)
    circuit.h(t3)
    circuit.sdg(t3)
    if c1 == 1:
        circuit.x(t1)
    if c2 == 1:
        circuit.x(t2)

# cyclomatic complexity 1 + 2 = 3
def add_end_toffoli(circuit: QuantumCircuit, t1: int, t2: int, t3: int, c1: int, c2: int, meas:int) -> None:
    if c1 == 1:
        circuit.x(t1)
    if c2 == 1:
        circuit.x(t2)
    circuit.h(t3)

    # circuit.measure(t3, meas)
    # circuit.cz(t1, t2)# .c_if(meas, 0)
    circuit.h(t2)
    circuit.ccx(t3,t1,t2)
    circuit.h(t2)

def create_qrom_without_succinct(control: int, target: int, val: int=0, data: str=None) -> QuantumCircuit:
    ancilla = control
    num_qubit = control + target + ancilla + 1
    qr = QuantumRegister(num_qubit, 'q')
    circuit = QuantumCircuit(qr)

    circuit.x(0)
    for ind in range(control):
        if val%2==1:
            circuit.x(1+ind*2)
        val//=2
    circuit.barrier()

    cur = 0
    for ind in range(control):
        add_toffoli(circuit, cur, cur+1, cur+2, 0, 1)
        cur += 2
    for step in range(2**control):
        for ind in range(target):
            if data is None:
                circuit.cx(cur, cur+1+ind)
            elif data[step]=='1':
                circuit.cx(cur, cur+1+ind)
        rev = 0
        step_sub = step
        while step_sub%2!=0:
            step_sub//=2
            rev+=1
        # print(step, rev)
        for ind in range(rev):
            add_toffoli(circuit, cur-2,cur-1,cur,0,0)
            cur -= 2
        if step+1 == 2**control:
            break
        circuit.cx(cur-2,cur)
        for ind in range(rev):
            add_toffoli(circuit, cur,cur+1,cur+2,0,1)
            cur += 2
    return circuit

# cyclomatic complexity 1 + 12(if) + 7(loop) + 5-1(add_toffoli) + 5-1(add_start_toffoli) + 3-1(add_end_toffoli) = 30
def create_qrom(control: int, target: int, succinct: bool, val: int=0, data: str=None) -> QuantumCircuit:
    ancilla = control
    num_qubit = control + target + ancilla + 1
    if succinct:
        qr = QuantumRegister(num_qubit, 'q')
        cr = ClassicalRegister(1, 'c')
        cr[0].name = "c0"
        circuit = QuantumCircuit(qr,cr)
    else:
        qr = QuantumRegister(num_qubit, 'q')
        circuit = QuantumCircuit(qr)

    circuit.x(0)
    for ind in range(control):
        if val%2==1:
            circuit.x(1+ind*2)
        val//=2
    circuit.barrier()

    cur = 0
    for ind in range(control):
        if succinct:
            add_start_toffoli(circuit, cur, cur+1, cur+2, 0, 1)
        else:
            add_toffoli(circuit, cur, cur+1, cur+2, 0, 1)
        cur += 2
    for step in range(2**control):
        for ind in range(target):
            if data is None:
                circuit.cx(cur, cur+1+ind)
            elif data[step]=='1':
                circuit.cx(cur, cur+1+ind)
        rev = 0
        step_sub = step
        while step_sub%2!=0:
            step_sub//=2
            rev+=1
        # print(step, rev)
        for ind in range(rev):
            if succinct:
                add_end_toffoli(circuit, cur-2,cur-1,cur,0,0,0)
            else:
                add_toffoli(circuit, cur-2,cur-1,cur,0,0)
            cur -= 2
        if step+1 == 2**control:
            break
        circuit.cx(cur-2,cur)
        for ind in range(rev):
            if succinct:
                add_start_toffoli(circuit, cur, cur+1, cur+2, 0, 1)
            else:
                add_toffoli(circuit, cur,cur+1,cur+2,0,1)
            cur += 2
    return circuit


def create_qrom_ref(control: int, target: int, succinct: bool, val: int=0, data: str=None) -> QuantumCircuit:
    ancilla = control
    num_qubit = target + (control + ancilla)*2
    if succinct:
        qr = QuantumRegister(num_qubit, 'q')
        cr = ClassicalRegister(1, 'c')
        cr[0].name = "c0"
        circuit = QuantumCircuit(qr,cr)
    else:
        qr = QuantumRegister(num_qubit, 'q')
        circuit = QuantumCircuit(qr)

    circuit.x(0)
    for ind in range(control):
        if val%2==1:
            circuit.x(1+ind*2)
        val//=2
    circuit.barrier()


    cur = 0
    rev_cur = num_qubit-1


    if succinct:
        add_start_toffoli(circuit, cur, cur+1, cur+2, 0, 1)
    else:
        add_toffoli(circuit, cur, cur+1, cur+2, 0, 1)
    if succinct:
        add_start_toffoli(circuit, cur, cur+1, rev_cur, 0, 0)
    else:
        add_toffoli(circuit, cur, cur+1, rev_cur, 0, 0)
    cur += 2
    for ind in range(control-1):
        circuit.cx(cur+1+ind*2, rev_cur-1-ind*2)
    circuit.barrier()

    for ind in range(1,control):
        if succinct:
            add_start_toffoli(circuit, cur, cur+1, cur+2, 0, 1)
        else:
            add_toffoli(circuit, cur, cur+1, cur+2, 0, 1)
        cur += 2
    for ind in range(1,control):
        if succinct:
            add_start_toffoli(circuit, rev_cur, rev_cur-1, rev_cur-2, 0, 1)
        else:
            add_toffoli(circuit, rev_cur, rev_cur-1, rev_cur-2, 0, 1)
        rev_cur -= 2
    circuit.barrier()

    for step in range(2**(control-1)):
        for ind in range(target):
            if data is None:
                circuit.cx(cur, cur+1+ind)
            elif data[step]=='1':
                circuit.cx(cur, cur+1+ind)

        for ind in range(target):
            if data is None:
                circuit.cx(rev_cur, cur+1+ind)
            elif data[step+2**(control-1)]=='1':
                circuit.cx(rev_cur, cur+1+ind)

        rev = 0
        step_sub = step
        while step_sub%2!=0:
            step_sub//=2
            rev+=1

        for ind in range(rev):
            if succinct:
                add_end_toffoli(circuit, cur-2,cur-1,cur,0,0,0)
            else:
                add_toffoli(circuit, cur-2,cur-1,cur,0,0)
            cur -= 2
            if succinct:
                add_end_toffoli(circuit, rev_cur+2, rev_cur+1,rev_cur,0,0,0)
            else:
                add_toffoli(circuit, rev_cur+2, rev_cur+1,rev_cur,0,0)
            rev_cur += 2

        if step+1 == 2**(control-1):
            break

        circuit.cx(cur-2,cur)
        circuit.cx(rev_cur+2,rev_cur)

        for ind in range(rev):
            if succinct:
                add_start_toffoli(circuit, cur, cur+1, cur+2, 0, 1)
            else:
                add_toffoli(circuit, cur,cur+1,cur+2,0,1)
            cur += 2
            if succinct:
                add_start_toffoli(circuit, rev_cur, rev_cur-1, rev_cur-2, 0, 1)
            else:
                add_toffoli(circuit, rev_cur, rev_cur-1, rev_cur-2, 0, 1)
            rev_cur -= 2
        circuit.barrier()


    circuit.barrier()
    cur = 0
    rev_cur = num_qubit-1
    if succinct:
        add_start_toffoli(circuit, cur, cur+1, rev_cur, 0, 0)
    else:
        add_toffoli(circuit, cur, cur+1, rev_cur, 0, 0)
    if succinct:
        add_start_toffoli(circuit, cur, cur+1, cur+2, 0, 1)
    else:
        add_toffoli(circuit, cur, cur+1, cur+2, 0, 1)
    cur += 2
    for ind in range(control-1):
        circuit.cx(cur+1+ind*2, rev_cur-1-ind*2)

    return circuit

