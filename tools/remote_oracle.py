"""Run oracle games remotely: on GitHub Actions (.github/workflows/oracle.yml),
or on a local remote runner if the main checkout has one
(porting/remote_runner.py, not in git; PTCG_REMOTE=actions forces Actions).

usage: remote_oracle.py <spec.json> <out_dir> [--start S] [--count N] [--shards K]
                        [--cov-files "sets/a.ts sets/b.ts"] [--tag T] [--ref BRANCH]

On Actions: dispatches the workflow with the gzip+base64 spec, waits for it,
downloads every shard's artifact and merges the traces into <out_dir>
(coverage snapshots, if requested, into <out_dir>/cov). The spec must fit in a
workflow input (~60 KB compressed; corpus specs are a few KB).
"""
import argparse, base64, calendar, glob, gzip, json, os, shutil, subprocess, sys, tempfile, time

REPO = os.environ.get('PTCG_REPO', 'christian-shin/ptcg-rust')


def gh(*args, capture=True):
    r = subprocess.run(['gh', *args], capture_output=capture, text=True)
    if r.returncode != 0:
        sys.exit('gh %s failed: %s' % (' '.join(args[:3]), (r.stderr or '').strip()[-500:]))
    return r.stdout


def local_runner():
    """The local remote runner module kept outside git, if this machine has one."""
    here = os.path.dirname(os.path.abspath(__file__))
    r = subprocess.run(['git', '-C', here, 'rev-parse', '--git-common-dir'], capture_output=True, text=True)
    common = os.path.normpath(os.path.join(here, r.stdout.strip()))   # relative to `here` unless absolute
    p = os.path.join(os.path.dirname(common), 'porting', 'remote_runner.py')
    return p if r.returncode == 0 and os.path.exists(p) else None


def run(spec_path, out, start=0, count=64, shards=8, cov_files='', tag='run', ref='oracle'):
    runner = local_runner()
    if runner and os.environ.get('PTCG_REMOTE') != 'actions':
        import importlib.util
        mspec = importlib.util.spec_from_file_location('remote_runner', runner)
        mod = importlib.util.module_from_spec(mspec)
        mspec.loader.exec_module(mod)
        return mod.run_oracle(spec_path, out, start=start, count=count, cov_files=cov_files, tag=tag)
    spec = json.load(open(spec_path))
    b64 = base64.b64encode(gzip.compress(json.dumps(spec, separators=(',', ':')).encode())).decode()
    if len(b64) > 60000:
        sys.exit('spec too large for a workflow input (%d bytes compressed)' % len(b64))
    shards = max(1, min(20, shards, count))
    # Unique tag: concurrent dispatches (several agents) are told apart by run title.
    tag = '%s-%s' % (tag, os.urandom(4).hex())
    t0 = time.time()
    gh('workflow', 'run', 'oracle.yml', '--repo', REPO, '--ref', 'main',
       '-f', 'spec_b64=' + b64, '-f', 'start=%d' % start, '-f', 'count=%d' % count,
       '-f', 'shards=%d' % shards, '-f', 'cov_files=' + cov_files, '-f', 'tag=' + tag, '-f', 'twinleaf_ref=' + ref)
    run_id = None
    for _ in range(30):
        time.sleep(3)
        runs = json.loads(gh('run', 'list', '--repo', REPO, '--workflow', 'oracle.yml', '--limit', '20',
                             '--json', 'databaseId,createdAt,status,displayTitle'))
        fresh = [r for r in runs if r.get('displayTitle') == 'oracle ' + tag
                 and calendar.timegm(time.strptime(r['createdAt'], '%Y-%m-%dT%H:%M:%SZ')) >= t0 - 5]
        if fresh:
            run_id = fresh[0]['databaseId']
            break
    if run_id is None:
        sys.exit('could not find the dispatched run')
    print('run https://github.com/%s/actions/runs/%s (%d games, %d shards)' % (REPO, run_id, count, shards), flush=True)
    subprocess.run(['gh', 'run', 'watch', str(run_id), '--repo', REPO, '--exit-status', '--interval', '15'],
                   stdout=subprocess.DEVNULL)
    status = json.loads(gh('run', 'view', str(run_id), '--repo', REPO, '--json', 'conclusion'))['conclusion']
    tmp = tempfile.mkdtemp(prefix='oracle-run-')
    gh('run', 'download', str(run_id), '--repo', REPO, '--dir', tmp, '--pattern', tag + '-shard*')
    os.makedirs(out, exist_ok=True)
    n = 0
    for f in glob.glob(os.path.join(tmp, '*', 'out', '*.json')):
        shutil.copy(f, out)
        n += 1
    cov = glob.glob(os.path.join(tmp, '*', 'covf', '*.json'))
    if cov:
        os.makedirs(os.path.join(out, 'cov'), exist_ok=True)
        for f in cov:
            shard = os.path.basename(os.path.dirname(os.path.dirname(f)))
            shutil.copy(f, os.path.join(out, 'cov', shard + '-' + os.path.basename(f)))
    logs = ''.join(open(f).read() for f in glob.glob(os.path.join(tmp, '*', 'log*.txt')))
    shutil.rmtree(tmp, ignore_errors=True)
    print('%s: %d traces, %d coverage snapshots in %.0fs' % (status, n, len(cov), time.time() - t0))
    return status, n, logs


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('spec')
    ap.add_argument('out')
    ap.add_argument('--start', type=int, default=0)
    ap.add_argument('--count', type=int, default=64)
    ap.add_argument('--shards', type=int, default=8)
    ap.add_argument('--cov-files', default='')
    ap.add_argument('--tag', default='run')
    ap.add_argument('--ref', default='oracle')
    a = ap.parse_args()
    status, n, logs = run(a.spec, a.out, a.start, a.count, a.shards, a.cov_files, a.tag, a.ref)
    bad = [l for l in logs.split('\n') if 'status=error' in l or 'status=stuck' in l or 'crashed' in l]
    for l in bad[:10]:
        print('ORACLE:', l)
    sys.exit(0 if status == 'success' else 1)


if __name__ == '__main__':
    main()
