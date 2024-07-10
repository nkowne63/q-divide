from .qrom import create_qrom, create_qrom_ref

import numpy as np
from qiskit import QuantumCircuit, Aer

def simulate(qasm_str):
    qc = QuantumCircuit.from_qasm_str(qasm_str)
    qc.save_statevector(label=f'state')
    simulator = Aer.get_backend('aer_simulator_statevector')
    vec = np.array(simulator.run(qc).result().data(0)["state"])
    # print(vec)
    # print(bin(np.argmax(vec)))
    return bin(np.argmax(vec))


def test():
    target = 1
    succinct = False
    for size in [2,3]:
        for val in range(0,2**size):
            for trial in range(10):
                data = "".join(np.random.choice(["0", "1"], 2**size))
                c = create_qrom(size, target, succinct, val=val, data=data)
                qasm = c.qasm()
                c_opt = create_qrom_ref(size, target, succinct, val=val, data=data)
                qasm_opt = c_opt.qasm()
                res1 = simulate(qasm)
                res2 = simulate(qasm_opt)
                print(f"size:{size}, val:{val}, trial:{trial}(data={data}) {res1==res2}")
                assert(res1 == res2)
