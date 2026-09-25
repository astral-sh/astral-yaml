#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = ["PyYAML==6.0.3"]
# ///
"""Check or regenerate syntax expectations with PyYAML, without running astral-yaml."""

import argparse
import json
from pathlib import Path

import yaml
from yaml import events, tokens


def event_text(event):
    if isinstance(event, events.StreamStartEvent):
        return "+STR"
    if isinstance(event, events.StreamEndEvent):
        return "-STR"
    if isinstance(event, events.DocumentStartEvent):
        return "+DOC" + (" ---" if event.explicit else "")
    if isinstance(event, events.DocumentEndEvent):
        return "-DOC" + (" ..." if event.explicit else "")
    if isinstance(event, events.SequenceEndEvent):
        return "-SEQ"
    if isinstance(event, events.MappingEndEvent):
        return "-MAP"
    if isinstance(event, events.AliasEvent):
        return "=ALI *" + event.anchor

    properties = ""
    if event.anchor is not None:
        properties += " &" + event.anchor
    if event.tag is not None:
        properties += " <" + event.tag + ">"

    if isinstance(event, events.SequenceStartEvent):
        return "+SEQ" + (" []" if event.flow_style else "") + properties
    if isinstance(event, events.MappingStartEvent):
        return "+MAP" + (" {}" if event.flow_style else "") + properties

    assert isinstance(event, events.ScalarEvent), event
    escapes = {"\\": "\\\\", "\n": "\\n", "\r": "\\r", "\t": "\\t", "\0": "\\0", "\b": "\\b"}
    value = "".join(escapes.get(char, char) for char in event.value)
    return "=VAL" + properties + " " + (event.style or ":") + value


def expectations(source, loader):
    event_output = "".join(
        event_text(event) + "\n" for event in yaml.parse(source, Loader=loader)
    )

    # PyYAML marks count Unicode characters; public AST spans count UTF-8 bytes.
    offsets = [0]
    for char in source:
        offsets.append(offsets[-1] + len(char.encode("utf-8")))

    scalars = []
    for token in yaml.scan(source, Loader=loader):
        if isinstance(token, tokens.ScalarToken):
            start = offsets[token.start_mark.index]
            end = offsets[token.end_mark.index]
            scalars.append(f"{start} {end}\n")

    return {"event": event_output, "scalars": "".join(scalars)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--write", action="store_true", help="replace committed expectations"
    )
    args = parser.parse_args()
    root = Path(__file__).resolve().parent.parent / "fixtures/real-world"

    for fixture in json.loads((root / "manifest.json").read_text()):
        path = root / fixture["file"]
        source = path.read_bytes().decode("utf-8")
        expected = expectations(source, yaml.BaseLoader)

        # Check both the Python parser and the LibYAML-backed parser. No schema
        # resolution occurs: plain `on`, for example, remains scalar text.
        assert expected == expectations(source, yaml.CBaseLoader), path

        for suffix, content in expected.items():
            snapshot = path.with_suffix("." + suffix)
            if args.write:
                snapshot.write_text(content)
            elif snapshot.read_text() != content:
                raise SystemExit(f"Oracle mismatch: {snapshot}")

        print(f"Verified {fixture['file']}")


if __name__ == "__main__":
    main()
