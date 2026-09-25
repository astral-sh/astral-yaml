#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.14"
# dependencies = ["PyYAML==6.0.3"]
# ///
"""Emit PyYAML syntax events and UTF-8 scalar ranges as JSON for the Rust tests."""

import json
from pathlib import Path
from typing import TypedDict

import yaml
from yaml import events, tokens


class Fixture(TypedDict):
    file: str


class Expectations(TypedDict):
    events: str
    scalars: list[tuple[int, int]]


class FixtureExpectations(Expectations):
    file: str


def event_text(event: events.Event) -> str:
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
        assert event.anchor is not None
        return "=ALI *" + event.anchor

    assert isinstance(
        event, (events.ScalarEvent, events.SequenceStartEvent, events.MappingStartEvent)
    ), event

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
    escapes = {
        "\\": "\\\\",
        "\n": "\\n",
        "\r": "\\r",
        "\t": "\\t",
        "\0": "\\0",
        "\b": "\\b",
    }
    value = "".join(escapes.get(char, char) for char in event.value)
    return "=VAL" + properties + " " + (event.style or ":") + value


def expectations(
    source: str, loader: type[yaml.BaseLoader | yaml.CBaseLoader]
) -> Expectations:
    event_output = "".join(
        event_text(event) + "\n" for event in yaml.parse(source, Loader=loader)
    )

    # PyYAML marks count Unicode characters; public AST spans count UTF-8 bytes.
    offsets = [0]
    for char in source:
        offsets.append(offsets[-1] + len(char.encode("utf-8")))

    scalars: list[tuple[int, int]] = []
    for token in yaml.scan(source, Loader=loader):
        if isinstance(token, tokens.ScalarToken):
            start = offsets[token.start_mark.index]
            end = offsets[token.end_mark.index]
            scalars.append((start, end))

    return {"events": event_output, "scalars": scalars}


def main() -> None:
    root = Path(__file__).resolve().parent.parent / "fixtures/real-world"
    fixtures: list[Fixture] = json.loads(
        (root / "manifest.json").read_text(encoding="utf-8")
    )
    results: list[FixtureExpectations] = []

    for fixture in fixtures:
        path = root / fixture["file"]
        source = path.read_bytes().decode("utf-8")
        expected = expectations(source, yaml.BaseLoader)

        # Check both the Python parser and the LibYAML-backed parser. No schema
        # resolution occurs: plain `on`, for example, remains scalar text.
        assert expected == expectations(source, yaml.CBaseLoader), path

        results.append({"file": fixture["file"], **expected})

    print(json.dumps(results))


if __name__ == "__main__":
    main()
