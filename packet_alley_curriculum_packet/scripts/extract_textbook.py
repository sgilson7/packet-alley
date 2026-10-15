#!/usr/bin/env python3
"""Deterministic online-source extractor for Packet Alley's openly licensed book.

Run from any working directory. Python stdlib only. It does not produce a passage
when its source/anchor changes, and never fabricates a substitute. Requires internet.

Usage:
  python3 scripts/extract_textbook.py
  python3 scripts/extract_textbook.py --fixture-dir tests/fixtures
"""
from __future__ import annotations

import argparse
import hashlib
import html
from html.parser import HTMLParser
import json
from pathlib import Path
import re
import sys
from datetime import datetime, timezone
from urllib.request import Request, urlopen
from urllib.error import URLError

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / 'data' / 'textbook_source_manifest.json'
OUTPUT = ROOT / 'data' / 'textbook_extracts.json'


class Paragraphs(HTMLParser):
    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.stack = []
        self.pieces = []
        self.out = []
        self.suppressed = 0

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        if tag in ('script', 'style'):
            self.suppressed += 1
        if tag in ('p', 'dd'):
            self.stack.append((tag, []))
        elif self.stack:
            # Notes/footnotes often appear as superscript links; omit such
            # superficial editorial decorations rather than deleting text.
            if tag == 'br':
                self.stack[-1][1].append(' ')

    def handle_data(self, data):
        if self.stack and not self.suppressed:
            # Include text in all active structural blocks (nested DD/P).
            for _, chunks in self.stack:
                chunks.append(data)

    def handle_endtag(self, tag):
        if tag in ('script', 'style') and self.suppressed:
            self.suppressed -= 1
        if self.stack and tag == self.stack[-1][0]:
            _, fragments = self.stack.pop()
            value = re.sub(r'\s+', ' ', html.unescape(''.join(fragments))).strip()
            if value:
                self.out.append(value)


def get_passage(paragraphs, anchor, max_sentences):
    # Authors put anchors at the start of the original paragraph; accepting
    # matches elsewhere risks silently selecting unrelated examples.
    candidates = list(dict.fromkeys(p for p in paragraphs if p.casefold().startswith(anchor.casefold())))
    if len(candidates) != 1:
        raise ValueError(f'Expected exactly one paragraph beginning {anchor!r}; found {len(candidates)}')
    original = candidates[0]
    # Split only at sentence punctuation followed by a capital; no paraphrase.
    # If prose contains unusual punctuation, verify resulting text manually.
    sentences = re.split(r'(?<=[.!?])\s+(?=[A-Z])', original)
    if len(sentences) < max_sentences:
        selected = original
    else:
        selected = ' '.join(sentences[:max_sentences])
    if not selected or len(selected) > 850:
        raise ValueError(f'Bad excerpt length {len(selected)} for {anchor!r}')
    return selected


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--fixture-dir', type=Path, help='Offline test fixture directory; filenames <ID>.html')
    args = parser.parse_args()
    manifest = json.loads(MANIFEST.read_text(encoding='utf-8'))
    received = {}
    failures = []
    for spec in manifest['passages']:
        id_, url = spec['id'], spec['url']
        try:
            if args.fixture_dir:
                raw = (args.fixture_dir / (id_ + '.html')).read_bytes()
            else:
                req = Request(url, headers={'User-Agent': 'PacketAlleyTextbookAttribution/1.0 (curriculum source audit)'})
                with urlopen(req, timeout=22) as resp:
                    raw = resp.read()
            parsed = Paragraphs()
            parsed.feed(raw.decode('utf-8'))
            selected = get_passage(parsed.out, spec['starts_with'], spec['max_sentences'])
            received[id_] = {
                'id': id_,
                'zone': spec['zone'],
                'verbatim': selected,
                'source_url': url,
                'title': manifest['work'],
                'attribution': manifest['authors'],
                'source_license_pending_per_file_verification': manifest['license'],
                'sha256': hashlib.sha256(selected.encode('utf-8')).hexdigest(),
                'extraction_method': 'HTML visible paragraph text; whitespace-only canonicalization; first N complete sentences',
            }
        except (OSError, URLError, UnicodeError, ValueError) as exc:
            failures.append(f'{id_}: {exc}')
    if failures:
        print('EXTRACTION FAILED: do not substitute or manufacture passages:', file=sys.stderr)
        for problem in failures:
            print(' - ' + problem, file=sys.stderr)
        print('No output written; review source files and verify licensing.', file=sys.stderr)
        return 2
    payload = {
        'work': manifest['work'],
        'book': manifest['book'],
        'extracted_utc': datetime.now(timezone.utc).isoformat(),
        'license_verification_required': True,
        'passages': received,
    }
    OUTPUT.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(f'Wrote {OUTPUT} with {len(received)} passages. Source-license review still required.')
    return 0

if __name__ == '__main__':
    raise SystemExit(main())
