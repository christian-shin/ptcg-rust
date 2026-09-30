"""Structural diff of two canonical states: statediff.py <oracle.json> <rust.json>

The oracle file may be the output of `cli.js state` ({"s": state, ...})."""
import json, sys

def load(p):
    v = json.load(open(p))
    return v['s'] if isinstance(v, dict) and 's' in v and 'players' in (v['s'] or {}) else v

def diff(a, b, path, out):
    if type(a) != type(b):
        out.append((path, a, b)); return
    if isinstance(a, dict):
        for k in sorted(set(a) | set(b)):
            if k not in a: out.append((path + '.' + k, '<absent>', b[k]))
            elif k not in b: out.append((path + '.' + k, a[k], '<absent>'))
            else: diff(a[k], b[k], path + '.' + k, out)
    elif isinstance(a, list):
        if len(a) != len(b) or any(not isinstance(x, (dict, list)) for x in a + b):
            if a != b: out.append((path, a, b))
            return
        for i, (x, y) in enumerate(zip(a, b)): diff(x, y, '%s[%d]' % (path, i), out)
    elif a != b:
        out.append((path, a, b))

out = []
diff(load(sys.argv[1]), load(sys.argv[2]), '', out)
for p, a, b in out:
    print('%s\n  oracle: %s\n  rust:   %s' % (p, json.dumps(a)[:300], json.dumps(b)[:300]))
print(len(out), 'differences')
