# astral-yaml for Python

`astral_yaml` reads and writes YAML 1.2 documents using Python's built-in
types. It supports CPython 3.11 and newer.

```python
import astral_yaml as yaml

config = yaml.loads("ports: [80, 443]\nmode: on")
assert yaml.dumps(config["mode"]) == '"on"\n'

config["backup_ports"] = config["ports"]
with open("config.yaml", "w", encoding="utf-8") as file:
    yaml.dump(config, file)

with open("config.yaml", "rb") as file:
    restored = yaml.load(file)
assert restored["ports"] is restored["backup_ports"]
```

Loading uses the YAML 1.2 core schema and detects UTF-8, UTF-16, and UTF-32
for binary input. An empty or comment-only stream returns `None`. Loading
rejects multiple documents, non-scalar mapping keys, and duplicate keys,
including keys Python considers equal, such as `true` and `1`.

The dumper accepts `None` and exact instances of `bool`, `int`, `float`, `str`,
`list`, `tuple`, and `dict`. Other types raise `TypeError`. Output uses flow
style with double-quoted strings and a trailing newline. Tuples become YAML
sequences, and aliases preserve shared and recursive containers.

Integers use Python's precision and its configured limit on decimal conversions.
Malformed input, unknown tags, and resource limits raise `astral_yaml.YAMLError`.
Error byte ranges use the original encoding for binary input and UTF-8 for text.
Reader and writer exceptions propagate to the caller.
