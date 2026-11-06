# astral-yaml for Python

`astral_yaml` reads and writes YAML 1.2 documents using Python's built-in
types. It supports CPython 3.11 and newer.

```python
import astral_yaml as yaml

config = yaml.loads("ports: [80, 443]\nmode: on")
assert config == {"ports": [80, 443], "mode": "on"}

config["ports"].append(8080)
with open("config.yaml", "w", encoding="utf-8") as file:
    yaml.dump(config, file)

with open("config.yaml", "rb") as file:
    assert yaml.load(file) == config
```

`loads` accepts text, bytes, and bytearrays. Binary input uses YAML's encoding
detection for UTF-8, UTF-16, and UTF-32. `load` accepts a text or binary reader.
An empty or comment-only stream returns `None`. A second document raises
`YAMLError`.

`dumps` returns a string in flow style with a trailing newline. Strings are
double-quoted so values like `"true"` round-trip as text. `dump` writes the
same text to a file-like object.

### Values and aliases

The binding reads the YAML 1.2 core schema into `None`, `bool`, `int`,
`float`, `str`, `list`, and `dict`. The dumper accepts these exact built-in
types and also writes tuples as YAML sequences.
Integers use Python's precision and its configured limit on decimal conversions.
Mapping keys must be scalars. Loading rejects duplicate keys, including keys
that Python would consider equal, such as `true` and `1`.

An alias resolves to the same object as its anchor. Dumping preserves shared
and recursive containers, including a list containing itself.

```python
import astral_yaml as yaml

ports = [443]
services = {"web": ports, "api": ports}
restored = yaml.loads(yaml.dumps(services))
assert restored["web"] is restored["api"]
```

Malformed input, unknown tags, and resource limits raise `astral_yaml.YAMLError`,
a subclass of `ValueError`. Byte ranges in parsing errors refer to the original
encoding for binary input and to UTF-8 for text input. Unsupported Python types
raise `TypeError`. Exceptions from the reader or writer propagate to the caller.
