"""Pack a golden corpus directory and (optionally) publish it as a GitHub release asset.

usage: golden_pack.py [DIR] [--out DIR] [--upload]

DIR defaults to corpus/golden/current, resolved to its real name (e.g. bf9d0ae). The archive
`<name>.tar.zst` (zstd -3; `.tar.gz` if there is no zstd) holds `spec.json` and `traces/` only.
--upload creates or updates the release `golden-<name>` on the `upstream` remote's repository (the tag
points at commit <name> if the repo has it, else at HEAD) and uploads the archive and SHA256SUMS.
CI's `golden` job downloads that asset (the name is in tools/golden-corpus.txt); after a re-record, run
this with --upload, write the new name into tools/golden-corpus.txt and update tools/golden-expected.txt.
"""
import argparse, hashlib, json, os, shutil, subprocess, sys, tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
REMOTE = os.environ.get('PTCG_CI_REMOTE', 'upstream')
KEEP = ['spec.json', 'traces']


def run(*cmd, **kw):
    r = subprocess.run(cmd, cwd=ROOT, text=True, capture_output=True, **kw)
    return r


def repo_slug():
    url = run('git', 'remote', 'get-url', REMOTE).stdout.strip()
    return url.removesuffix('.git').split('github.com')[-1].lstrip(':/')


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('dir', nargs='?', default=os.path.join(ROOT, 'corpus', 'golden', 'current'))
    ap.add_argument('--out', default=os.environ.get('PTCG_GOLDEN_OUT') or tempfile.gettempdir())
    ap.add_argument('--upload', action='store_true')
    a = ap.parse_args()
    src = os.path.realpath(a.dir)
    name = os.path.basename(src)
    for k in KEEP:
        if not os.path.exists(os.path.join(src, k)):
            sys.exit('%s has no %s' % (src, k))
    extra = [e for e in os.listdir(src) if e not in KEEP]
    if extra:
        print('not packed (only %s are): %s' % (', '.join(KEEP), ', '.join(extra)))
    os.makedirs(a.out, exist_ok=True)
    if shutil.which('zstd'):
        arc = os.path.join(a.out, name + '.tar.zst')
        tar = subprocess.Popen(['tar', '-C', src, '-cf', '-'] + KEEP, stdout=subprocess.PIPE)
        with open(arc, 'wb') as f:
            z = subprocess.run(['zstd', '-3', '-T0', '-q', '-f'], stdin=tar.stdout, stdout=f)
        tar.wait()
        ok = z.returncode == 0 and tar.returncode == 0
    else:
        arc = os.path.join(a.out, name + '.tar.gz')
        ok = subprocess.run(['tar', '-C', src, '-czf', arc] + KEEP).returncode == 0
    if not ok:
        sys.exit('packing failed')
    h = hashlib.sha256()
    with open(arc, 'rb') as f:
        for chunk in iter(lambda: f.read(1 << 20), b''):
            h.update(chunk)
    sha = h.hexdigest()
    sums = os.path.join(a.out, 'SHA256SUMS')
    with open(sums, 'w') as f:
        f.write('%s  %s\n' % (sha, os.path.basename(arc)))
    print('%s  %d bytes (%.1f MB)  sha256 %s' % (arc, os.path.getsize(arc), os.path.getsize(arc) / 1e6, sha))
    if not a.upload:
        return
    repo = repo_slug()
    tag = 'golden-' + name
    target = name if run('git', 'cat-file', '-e', name + '^{commit}').returncode == 0 else 'HEAD'
    if target == 'HEAD':
        target = run('git', 'rev-parse', 'HEAD').stdout.strip()
    else:
        target = run('git', 'rev-parse', name + '^{commit}').stdout.strip()
    if run('gh', 'release', 'view', tag, '-R', repo).returncode != 0:
        r = run('gh', 'release', 'create', tag, '-R', repo, '--target', target,
                '--title', 'Golden corpus ' + name,
                '--notes', 'Golden-corpus replay traces (spec.json + traces/), recorded at %s. SHA256SUMS holds the archive digest.' % name)
        if r.returncode != 0:
            sys.exit(r.stderr)
    r = run('gh', 'release', 'upload', tag, arc, sums, '-R', repo, '--clobber')
    if r.returncode != 0:
        sys.exit(r.stderr)
    print('https://github.com/%s/releases/tag/%s' % (repo, tag))


if __name__ == '__main__':
    main()
