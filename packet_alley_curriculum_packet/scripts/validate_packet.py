#!/usr/bin/env python3
"""Static consistency checks for authored prose and build contracts. Does NOT test live game."""
from __future__ import annotations
from pathlib import Path
import json
import re
import sys

ROOT = Path(__file__).resolve().parents[1]
D = ROOT/'data'
expected = {
 'block': ['address','hub','loop','crowd','backbone','floor'],
 'town': ['gateway','route','ttl','point','roads','across'],
 'wires': ['arp','handshake','transfer','window','connect','reliable'],
 'storm': ['flood'],
}
errs=[]

def assert_(condition, message):
    if not condition: errs.append(message)

try:
    story = json.loads((D/'verbatim_story_copy.json').read_text(encoding='utf-8'))
    blueprints = json.loads((D/'level_blueprints.json').read_text(encoding='utf-8'))
    sources = json.loads((D/'textbook_source_manifest.json').read_text(encoding='utf-8'))
except (OSError, ValueError) as e:
    print('Cannot open/read packet JSON:', e, file=sys.stderr)
    raise SystemExit(2)

ids=set(sum(expected.values(), []))
assert_(set(story['levels'])==ids, 'Story level IDs mismatch')
assert_(set(blueprints['levels'])==ids, 'Blueprint level IDs mismatch')
assert_(set(blueprints['zones'])==set(expected), 'Blueprint zones mismatch')
assert_(set(story['chapters'])==set(expected), 'Story chapters mismatch')

for id_ in ids:
    s=story['levels'].get(id_,{})
    b=blueprints['levels'].get(id_,{})
    for k in ('title', 'story','task','debrief', 'inspect_prompt', 'predict_prompt'):
        assert_(isinstance(s.get(k),str) and len(s[k])>5, f'{id_}: no {k}')
    for k in ('revised_objective','evidence_to_inspect','commit_prediction','map_interaction'):
        assert_(bool(b.get(k)),f'{id_}: no {k}')
    assert_(b.get('chapter') == next((z for z,v in expected.items() if id_ in v), None),f'{id_}: wrong chapter')
    # The authored task must not prescribe the physical or numeric answer.
    task=s.get('task','').lower()
    for forbidden in ('swap the hub for a switch','block the spare','correct choice is','choose option 1','answer is','the right route is'):
        assert_(forbidden not in task, f'{id_}: answer leak phrase: {forbidden}')

source_ids=[x['id'] for x in sources['passages']]
assert_(len(source_ids)==len(set(source_ids)), 'Duplicated source ID')
assert_(len(source_ids)>=11,'Too few source excerpts')
for sp in sources['passages']:
    assert_(sp['url'].startswith('https://beta.computer-networking.info/'),f'Bad URL {sp["id"]}')
    assert_(len(sp['starts_with'])>18,f'Weak anchor {sp["id"]}')
    assert_(sp['max_sentences'] in (1,2,3),f'Unbounded excerpt {sp["id"]}')

for z in expected:
    assert_(story['chapters'][z].get('opening'), f'Zone missing opening {z}')
    assert_(story['chapters'][z].get('example_instruction'), f'Zone missing interactive example {z}')
    assert_(len([p for p in sources['passages'] if p['zone']==z]) >=2, f'Zone without textbook passages: {z}')

extracts=D/'textbook_extracts.json'
if extracts.exists():
    data=json.loads(extracts.read_text(encoding='utf-8'))
    assert_(set(data['passages'])==set(source_ids),'Extracted passage IDs mismatch')
    assert_(all(isinstance(x['verbatim'],str) and x['sha256'] for x in data['passages'].values()), 'Invalid extracts')
    print('Verbatim extract file found; still requires checking original page and license.')
else:
    print('NOTICE: textbook_extracts.json not generated; source extraction awaits agent with web access.')

if errs:
    print('PACKET VALIDATION FAILED')
    for err in errs: print(' -',err)
    raise SystemExit(1)
print(f'Packet structure OK: {len(ids)} levels; {len(expected)} zone intros; {len(source_ids)} textbook passage anchors.')
print('These checks DO NOT verify repo integration, instructor signoff, or pedagogy in a playable browser.')
