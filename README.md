# Q-DIVIDE

SELECT circuit implementation in Qiskit is [create_qrom](./pyimpl/qrom.py), which cyclomatic complexity is 30.

Implementation in Q-DIVIDE is [uniform_layered_internal](./src/pyfunctions/internal.rs), which cyclomatic complexity is 17.

Both complexities include that of subsequent function calls, and are counted in terms of elementary gates.

# how to call rust functions in python

## setup

You should first enter the python virtual environment.

```sh
$ pipenv shell
$ pipenv install
```

Run the following commands to build rust module:
`maturin develop`

then you can use package

```python
import prepare_circuit
```
