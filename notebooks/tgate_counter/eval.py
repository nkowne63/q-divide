from .qrom import create_qrom, create_qrom_ref
from .counter import count_tgate

def evaluate_qrom(start: int, end: int) -> None:
    for size in range(start,end+1):
        target = 1
        c = create_qrom(size, target, True)
        c_opt = create_qrom_ref(size, target, True)
        qasm = c.qasm()
        qasm_opt = c_opt.qasm()
        res = count_tgate(qasm)
        res_opt = count_tgate(qasm_opt)
        print(f"size:{size:2} "
            f"t-count: {res[0]:4}->{res_opt[0]:4} ({res_opt[0]/res[0]*100:.2f}%) "
            f"t-depth: {res[1]:4}->{res_opt[1]:4} ({res_opt[1]/res[1]*100:.2f}%) ")
        