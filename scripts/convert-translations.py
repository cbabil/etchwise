#!/usr/bin/env python3
"""Converts upstream Qt Linguist files (src/i18n/rpi-imager_*.ts) into JSON
catalogues for the React UI: {context: {source: translation}}.

Unfinished (empty), vanished and obsolete messages are skipped, as Qt's
lrelease does; plural forms use the first form.

Usage: convert-translations.py <upstream src/i18n dir> <output dir>
"""
import json
import pathlib
import sys
import xml.etree.ElementTree as ET


def convert(ts_file: pathlib.Path) -> dict:
    catalogue: dict = {}
    for context in ET.parse(ts_file).getroot().iter("context"):
        name = context.findtext("name", "")
        for message in context.iter("message"):
            source = message.findtext("source")
            node = message.find("translation")
            if source is None or node is None or node.get("type") in ("vanished", "obsolete"):
                continue
            forms = node.findall("numerusform")
            text = forms[0].text if forms else node.text
            if text:
                catalogue.setdefault(name, {})[source] = text
    return catalogue


def main() -> None:
    source_dir, out_dir = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
    out_dir.mkdir(parents=True, exist_ok=True)
    for ts_file in sorted(source_dir.glob("rpi-imager_*.ts")):
        lang = ts_file.stem.removeprefix("rpi-imager_")
        data = convert(ts_file)
        (out_dir / f"{lang}.json").write_text(json.dumps(data, ensure_ascii=False, indent=1, sort_keys=True) + "\n")
        print(lang, sum(len(v) for v in data.values()))


if __name__ == "__main__":
    main()
