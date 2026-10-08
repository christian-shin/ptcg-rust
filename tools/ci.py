"""Run the heavy checks on GitHub Actions instead of this machine.

usage: ci.py [--name NAME] [--games N] [--shards K] [--no-wait]

Pushes the current commit to the `ci/<NAME>` branch (default: the current
branch name), which starts the `rust` workflow (.github/workflows/rust.yml):
build, `cargo test`, every scenario, and `tools/fuzz.py` split across K
runners (default 4 x 25,000 games). With --games/--shards it dispatches the
workflow on that branch instead of relying on the push. Then it waits for the
run, prints each job's result and, on failure, downloads the failing games'
traces to corpus/ci/<run id>/ (replay them with engine/target/release/diff)
and prints the failed steps' log tail. Exit status 0 only when the run passed.

Commit first: only committed work is pushed. The ci/ branches are scratch
branches; each push replaces the previous one of the same name.
"""
import argparse, json, os, subprocess, sys, time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
REMOTE = os.environ.get('PTCG_CI_REMOTE', 'upstream')
WORKFLOW = 'rust.yml'


def sh(*cmd, check=True, capture=True):
    r = subprocess.run(cmd, cwd=ROOT, text=True, capture_output=capture)
    if check and r.returncode != 0:
        sys.exit('%s failed:\n%s%s' % (' '.join(cmd), r.stdout or '', r.stderr or ''))
    return r


def find_run(branch, sha, since):
    for _ in range(60):
        r = sh('gh', 'run', 'list', '--workflow', WORKFLOW, '--branch', branch, '--limit', '10',
               '--json', 'databaseId,headSha,createdAt,status,event')
        for run in json.loads(r.stdout or '[]'):
            if run['headSha'] == sha and run['createdAt'] >= since:
                return run['databaseId']
        time.sleep(5)
    sys.exit('no %s run appeared for %s on %s' % (WORKFLOW, sha[:9], branch))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--name')
    ap.add_argument('--games', type=int)
    ap.add_argument('--shards', type=int)
    ap.add_argument('--no-wait', action='store_true')
    a = ap.parse_args()
    if sh('git', 'status', '--porcelain', '--untracked-files=no').stdout.strip():
        print('note: uncommitted changes are not part of the run')
    name = a.name or sh('git', 'rev-parse', '--abbrev-ref', 'HEAD').stdout.strip()
    branch = 'ci/' + name.replace('/', '-')
    sha = sh('git', 'rev-parse', 'HEAD').stdout.strip()
    since = time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime(time.time() - 30))
    sh('git', 'push', '--force', '--quiet', REMOTE, 'HEAD:refs/heads/' + branch)
    print('pushed %s to %s' % (sha[:9], branch))
    if a.games or a.shards:
        args = ['gh', 'workflow', 'run', WORKFLOW, '--ref', branch]
        if a.games:
            args += ['-f', 'games=%d' % a.games]
        if a.shards:
            args += ['-f', 'shards=%d' % a.shards]
        sh(*args)
        # The push started a default run too; the dispatched one is the one to follow.
        time.sleep(3)
    run = find_run(branch, sha, since)
    if a.games or a.shards:
        r = sh('gh', 'run', 'list', '--workflow', WORKFLOW, '--branch', branch, '--event', 'workflow_dispatch',
               '--limit', '5', '--json', 'databaseId,headSha')
        runs = [x['databaseId'] for x in json.loads(r.stdout or '[]') if x['headSha'] == sha]
        run = runs[0] if runs else run
    url = sh('gh', 'run', 'view', str(run), '--json', 'url', '-q', '.url').stdout.strip()
    print('run %s: %s' % (run, url))
    if a.no_wait:
        return
    sh('gh', 'run', 'watch', str(run), '--exit-status', '--interval', '30', check=False, capture=True)
    view = json.loads(sh('gh', 'run', 'view', str(run), '--json', 'conclusion,jobs').stdout)
    for j in view['jobs']:
        print('  %-12s %s' % (j['name'], j['conclusion']))
    if view['conclusion'] == 'success':
        print('PASSED')
        return
    out = os.path.join(ROOT, 'corpus', 'ci', str(run))
    os.makedirs(out, exist_ok=True)
    sh('gh', 'run', 'download', str(run), '--dir', out, check=False)
    print('artifacts (failing traces, if any) in %s' % os.path.relpath(out, ROOT))
    log = sh('gh', 'run', 'view', str(run), '--log-failed', check=False).stdout
    print('\n'.join(log.splitlines()[-60:]))
    print('FAILED')
    sys.exit(1)


if __name__ == '__main__':
    main()
