# Q-Divide

## Cyclomatic complexity

SELECT circuit implementation in Qiskit is [create_qrom](./pyimpl/qrom.py#L110), which cyclomatic complexity is 30.

Implementation in Q-Divide is [uniform_layered_internal](./src/pyfunctions/internal.rs#L7), which cyclomatic complexity is 17.

Both complexities include that of subsequent function calls, and both are counted in terms of elementary gates.

## How to call rust functions in python


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
