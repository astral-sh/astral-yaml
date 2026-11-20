# astral-yaml for Python

`astral_yaml` reads and writes YAML 1.2 on CPython 3.11 and newer.

## Reading YAML

The loader uses the YAML 1.2 core schema. In this configuration, the port
numbers become Python integers and the unquoted word `on` stays a string.

```python
import astral_yaml as yaml

config = yaml.loads("ports: [80, 443]\nmode: on")
assert config == {"ports": [80, 443], "mode": "on"}
```

`load` and `loads` accept at most one document per call. Empty or
comment-only input returns `None`. A second document raises `yaml.YAMLError`.

Use `load_all` for an open text or binary file and `loads_all` for a string
or encoded bytes. Both return a list in document order. An empty or
comment-only stream gives `[]`, and an explicit empty document contributes
`None`. An error in any document raises during the call.

```python
configs = yaml.loads_all("---\nmode: on\n---\nports: [80, 443]\n")
assert configs == [{"mode": "on"}, {"ports": [80, 443]}]
```

Mapping keys must be scalars and must stay unique when converted to Python.
For example, `{true: first, 1: second}` raises `yaml.YAMLError` because
`True == 1` in Python.

## Writing YAML

`dumps` returns YAML as a string, and `dump` writes it to a text stream.
Both use block style by default, with two spaces per indentation level.
Pass `style="flow"` for collections written with brackets and braces.
Empty collections use `[]` or `{}` in either style.

```python
config["ports"].append(8080)
assert yaml.dumps(config) == '"ports":\n  - 80\n  - 443\n  - 8080\n"mode": "on"\n'
assert yaml.dumps(config, style="flow") == '{"ports": [80, 443, 8080], "mode": "on"}\n'

with open("config.yaml", "w", encoding="utf-8") as file:
    yaml.dump(config, file)

# In binary mode, load detects UTF-8, UTF-16, or UTF-32.
with open("config.yaml", "rb") as file:
    assert yaml.load(file) == config
```

Tuples serialize as sequences and load back as lists. If two dictionary
entries share a list, the dumper uses a YAML alias so the entries still
share a list when you read them back. Aliases also preserve cycles.

## Errors

- **YAML errors.** Invalid YAML, unsupported tags, and resource limits
  (including Python's digit limit for decimal integers) raise `yaml.YAMLError`.
  Parse errors report byte offsets in the original encoding for binary input
  and in UTF-8 for text.
- **Unsupported Python values.** The dumper accepts `None` and exact instances
  of `bool`, `int`, `float`, `str`, `list`, `tuple`, and `dict`. Other types
  raise `TypeError`.
- **I/O errors.** Exceptions from the file's `read` or `write` method
  propagate to the caller.
