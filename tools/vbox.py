"""Verification box: oracle, tier-4, regen and ad-hoc runs on our EC2 instance.

Runs on the working tree, not on git refs: the Twinleaf source (PTCG_ORACLE,
default twinleaf/ptcg-server) and this repo's engine/data/decks/tools are
synced to the box as they are on disk, committed or not, and built there once
per content hash. Jobs run detached on the box (several at once; tier 4 and
regen at lower priority) and their results are copied back.

usage: vbox.py up | stop [--force] | ssh | setup | gc
       vbox.py oracle SPEC OUT [--start S] [--count N] [--procs P] [--cov-files "sets/a.ts ..."]
       vbox.py tier4 [--games N] [--shards K] [--seed S] [--chunk C] [--detach]
       vbox.py regen HEADERS.json.gz OUT [--shards K]
       vbox.py exec [--out DIR] [--no-oracle] -- CMD ...   (runs in the repo snapshot;
               $PTCG_ORACLE, $PTCG_DIFF, $JOB set; files written to $JOB/out are fetched)
       vbox.py ledger [--gate 100000]
       vbox.py status [JOB] | logs JOB [--tail N] | wait JOB | fetch JOB DEST | cancel JOB

Oracle-side flags take --oracle DIR (a ptcg-server directory). The tier-4
ledger counts green games per (engine/data/decks trees, Twinleaf source hash),
so it resets by itself whenever either side changes. tools/check_cards.py
--remote and tools/tier4.py --remote use the box unless PTCG_REMOTE=actions.

The box stops itself after 30 idle minutes; every command starts it again.
Setup: tools/vbox_setup.sh (`vbox.py setup`). Instance and credentials:
PTCG_VBOX_INSTANCE, PTCG_VBOX_PROFILE (AWS CLI profile), PTCG_VBOX_REGION,
PTCG_VBOX_KEY (SSH key).
"""
import argparse, contextlib, fcntl, glob, hashlib, json, os, re, shlex, shutil, signal, subprocess, sys, tempfile, time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
INSTANCE = os.environ.get('PTCG_VBOX_INSTANCE', 'i-0aa8823823cdc826f')
PROFILE = os.environ.get('PTCG_VBOX_PROFILE', 'ptcg')
REGION = os.environ.get('PTCG_VBOX_REGION', 'us-west-2')
KEY = os.path.expanduser(os.environ.get('PTCG_VBOX_KEY', '~/.ssh/ptcg_vbox'))
CACHE = os.path.expanduser('~/.cache/vbox')
RSYNC = next((p for p in ('/opt/homebrew/bin/rsync', '/usr/local/bin/rsync') if os.path.exists(p)), 'rsync')
SSH_OPTS = ['-i', KEY, '-o', 'HostKeyAlias=ptcg-vbox', '-o', 'UserKnownHostsFile=~/.ssh/ptcg_vbox_known_hosts',
            '-o', 'StrictHostKeyChecking=accept-new', '-o', 'ControlMaster=auto', '-o', 'ControlPath=~/.ssh/cm-vbox-%C',
            '-o', 'ControlPersist=120', '-o', 'ServerAliveInterval=30', '-o', 'ConnectTimeout=10', '-o', 'BatchMode=yes']
# What each build is made of (paths relative to the source directory).
ORACLE_PATHS = ['src', 'package.json', 'package-lock.json', 'tsconfig.json']
REPO_PATHS = ['engine', 'data', 'decks', 'tools', 'scenarios', 'divergences.toml']
GATE_TREES = ('engine', 'data', 'decks')   # = tools/tier4_ledger.py code_key
# Bump on every change to the agent half: clients from older worktrees then
# leave the newer agent on the box in place (it is installed as vbox/agent.py).
AGENT_VERSION = 2


def die(msg):
    sys.exit('vbox: ' + msg)


# ---------------------------------------------------------------- client: box

def aws(*args):
    r = subprocess.run(['aws', *args, '--profile', PROFILE, '--region', REGION, '--output', 'json'],
                       capture_output=True, text=True)
    if r.returncode:
        die('aws %s: %s' % (args[1], r.stderr.strip()[-400:]))
    return json.loads(r.stdout or 'null')


def box_state():
    i = aws('ec2', 'describe-instances', '--instance-ids', INSTANCE)['Reservations'][0]['Instances'][0]
    return i['State']['Name'], i.get('PublicIpAddress')


_ip = None


def ensure_up(refresh=False):
    """Start the instance if needed; return its IP once SSH answers."""
    global _ip
    if _ip and not refresh:
        return _ip
    t0, said = time.time(), False
    while time.time() - t0 < 600:
        state, ip = box_state()
        if state == 'running' and ip:
            r = subprocess.run(['ssh', *SSH_OPTS, 'ubuntu@' + ip, 'mkdir -p vbox && touch vbox/activity && (cat vbox/agent.version 2>/dev/null || true)'],
                               capture_output=True, text=True)
            if r.returncode == 0:
                _ip = ip
                if int(r.stdout.strip() or 0) < AGENT_VERSION:
                    subprocess.run([RSYNC, '-a', '-e', ssh_cmd(), os.path.abspath(__file__), 'ubuntu@%s:vbox/agent.py.new' % ip], check=True)
                    subprocess.run(['ssh', *SSH_OPTS, 'ubuntu@' + ip,
                                    'mv vbox/agent.py.new vbox/agent.py && echo %d > vbox/agent.version' % AGENT_VERSION], check=True)
                return ip
        elif state == 'stopped':
            aws('ec2', 'start-instances', '--instance-ids', INSTANCE)
        elif state in ('terminated', 'shutting-down'):
            die('instance %s is %s' % (INSTANCE, state))
        if not said:
            print('vbox: starting the box (%s)...' % state, file=sys.stderr, flush=True)
            said = True
        time.sleep(5)
    die('the box did not come up within 10 minutes')


def ssh_cmd():
    return ' '.join(['ssh'] + [shlex.quote(o) for o in SSH_OPTS])


def ssh(cmd, input=None, check=True, capture=True):
    """Run a shell command on the box (retrying once if the connection drops)."""
    for attempt in range(3):
        ip = ensure_up(refresh=attempt > 0)
        r = subprocess.run(['ssh', *SSH_OPTS, 'ubuntu@' + ip, cmd], input=input, text=True,
                           capture_output=capture)
        if r.returncode != 255:
            break
        time.sleep(5)
    if check and r.returncode:
        die('on the box: %s\n%s' % (cmd[:200], ((r.stdout or '') + (r.stderr or '')).strip()[-3000:]))
    return r


def agent(*args, input=None, check=True, capture=True):
    return ssh('python3 vbox/agent.py agent ' + ' '.join(shlex.quote(str(a)) for a in args), input=input, check=check,
               capture=capture)


def rsync_to(srcs, dest, *flags):
    ip = ensure_up()
    subprocess.run([RSYNC, '-az', *flags, '-e', ssh_cmd(), *srcs, 'ubuntu@%s:%s' % (ip, dest)], check=True)


def rsync_from(src, dest, *flags):
    ip = ensure_up()
    os.makedirs(dest, exist_ok=True)
    subprocess.run([RSYNC, '-az', *flags, '-e', ssh_cmd(), 'ubuntu@%s:%s' % (ip, src), dest], check=True)


# ------------------------------------------------------------ client: builds

def git(cwd, *args, env=None):
    r = subprocess.run(['git', '-C', cwd, *args], capture_output=True, text=True, env=env)
    if r.returncode:
        die('git %s in %s: %s' % (args[0], cwd, r.stderr.strip()))
    return r.stdout


def snapshot(src, paths):
    """Content hash of `paths` under `src` as they are on disk (tracked files
    with their edits, untracked files, minus ignored ones), the file list, and
    each path's git tree hash."""
    present = [p for p in paths if os.path.exists(os.path.join(src, p))]
    prefix = git(src, 'rev-parse', '--show-prefix').strip()
    index = os.path.join(src, git(src, 'rev-parse', '--git-path', 'index').strip())
    with tempfile.TemporaryDirectory() as t:
        tmp = os.path.join(t, 'index')
        shutil.copy(index, tmp)   # a copy keeps the stat cache: only edited files are rehashed
        env = dict(os.environ, GIT_INDEX_FILE=tmp)
        git(src, 'add', '-A', '--', *present, env=env)
        tree = git(src, 'write-tree', env=env).strip()
        subs = git(src, 'rev-parse', *['%s:%s%s' % (tree, prefix, p) for p in present]).split()
        files = git(src, 'ls-files', '-z', '--cached', '--', *present, env=env).split('\0')
    trees = dict(zip(present, subs))
    h = hashlib.sha1(''.join('%s %s\n' % kv for kv in sorted(trees.items())).encode()).hexdigest()[:16]
    return h, [f for f in files if f], trees


@contextlib.contextmanager
def flock(path):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, 'w') as f:
        fcntl.flock(f, fcntl.LOCK_EX)
        yield


def ensure_build(kind, src, paths):
    """Sync `src` to the box and build it there unless that content is built; return (hash, trees)."""
    h, files, trees = snapshot(src, paths)
    slot = '%s-%s' % (kind, hashlib.sha1(os.path.realpath(src).encode()).hexdigest()[:10])
    with flock(os.path.join(CACHE, slot + '.lock')):   # one sync+build per local tree at a time
        if agent('has', kind, h).stdout.strip() == 'yes':
            return h, trees
        print('vbox: syncing %s (%s %s) and building it on the box...' % (os.path.relpath(src), kind, h), file=sys.stderr, flush=True)
        with tempfile.NamedTemporaryFile('w', suffix='.txt') as fl:
            fl.write('\n'.join(files))
            fl.flush()
            rsync_to([src + '/'], 'stage/%s/' % slot, '--files-from=' + fl.name)
        r = agent('build', kind, slot, h, input='\0'.join(files), check=False)
        if r.returncode:
            die('%s build failed:\n%s' % (kind, (r.stdout + r.stderr)[-4000:]))
    return h, trees


def default_oracle():
    """PTCG_ORACLE; else the Twinleaf worktree on branch PTCG_ORACLE_REF; else twinleaf/ptcg-server."""
    if os.environ.get('PTCG_ORACLE'):
        return os.path.abspath(os.environ['PTCG_ORACLE'])
    main = os.path.dirname(os.path.abspath(git(ROOT, 'rev-parse', '--git-common-dir').strip()))
    tl = os.path.join(main, 'twinleaf')
    ref = os.environ.get('PTCG_ORACLE_REF')
    if ref and ref != 'oracle':
        wt = git(tl, 'worktree', 'list', '--porcelain')
        for block in wt.split('\n\n'):
            if 'branch refs/heads/%s\n' % ref in block + '\n':
                return os.path.join(block.split('\n')[0].split(' ', 1)[1], 'ptcg-server')
        die('PTCG_ORACLE_REF=%s: no Twinleaf worktree has that branch checked out; set PTCG_ORACLE to its ptcg-server' % ref)
    return os.path.join(tl, 'ptcg-server')


def repo_build():
    h, trees = ensure_build('repo', ROOT, REPO_PATHS)
    return h, '-'.join(trees[t][:9] for t in GATE_TREES)


# -------------------------------------------------------------- client: jobs

def submit(kind, cmds, oracle=None, repo=None, nice=0, inputs=None, post=None, meta=None, timeout=86400):
    jid = '%s-%s-%s' % (time.strftime('%Y%m%d-%H%M%S'), kind, os.urandom(2).hex())
    ssh('mkdir -p jobs/%s/in jobs/%s/out' % (jid, jid))
    for name, path in (inputs or {}).items():
        rsync_to([path], 'jobs/%s/in/%s' % (jid, name))
    job = {'id': jid, 'kind': kind, 'oracle': oracle, 'repo': repo, 'nice': nice, 'cmds': cmds, 'post': post or [],
           'meta': meta or {}, 'timeout': timeout, 'created': time.time(),
           'client': {'root': ROOT, 'oracle_dir': (meta or {}).get('oracle_dir')}}
    agent('start', jid, input=json.dumps(job))
    return jid


def status(jid):
    return json.loads(agent('status', jid).stdout)


def wait(jid, verbose=True):
    t0, last, shown = time.time(), 0, {}
    try:
        while True:
            st = status(jid)
            if st['state'] not in ('queued', 'running'):
                return st
            if verbose and time.time() - last >= 60:
                last = time.time()
                new = [l for l in st.get('progress', []) if l and shown.get(l) is None]
                for l in new[:20]:
                    shown[l] = True
                    print('  ' + l, flush=True)
                if not new:
                    print('  [%s running, %.0f min]' % (jid, (time.time() - t0) / 60), flush=True)
            time.sleep(10)
    except KeyboardInterrupt:
        print('\nvbox: job %s keeps running on the box; `vbox.py wait %s` or `vbox.py cancel %s`' % (jid, jid, jid))
        sys.exit(130)


def logs(jid, tail=0):
    return agent('logs', jid, tail).stdout


def run_oracle(spec_path, out, start=0, count=64, cov_files='', oracle_dir=None, procs=None, tag='run', **_):
    """Play oracle games on the box (same contract as remote_oracle.run):
    traces into `out`, filtered coverage into out/cov; returns (status, n, logs)."""
    oracle_dir = oracle_dir or default_oracle()
    t0 = time.time()
    oh, _ = ensure_build('oracle', oracle_dir, ORACLE_PATHS)
    procs = max(1, min(procs or 48, count))
    per = -(-count // procs)
    cmds = []
    for j in range(procs):
        a, c = start + j * per, min(per, start + count - (start + j * per))
        if c <= 0:
            break
        cov = 'mkdir -p $JOB/covraw/%d && NODE_V8_COVERAGE=$JOB/covraw/%d ' % (j, j) if cov_files else ''
        cmds.append({'name': 'p%02d' % j, 'cwd': 'oracle',
                     'sh': '%snode --max-old-space-size=3072 output/oracle/cli.js corpus $JOB/in/spec.json $JOB/out %d %d' % (cov, a, c)})
    post = [{'builtin': 'cov-filter', 'files': cov_files.split()}] if cov_files else []
    jid = submit('oracle', cmds, oracle=oh, inputs={'spec.json': spec_path}, post=post,
                 meta={'tag': tag, 'start': start, 'count': count, 'oracle_dir': oracle_dir})
    print('vbox: job %s (%d games, %d processes, oracle %s)' % (jid, count, len(cmds), oh), flush=True)
    st = wait(jid, verbose=False)
    rsync_from('jobs/%s/out/' % jid, out)
    n = len(glob.glob(os.path.join(out, '*.json')))
    ncov = len(glob.glob(os.path.join(out, 'cov', '*.json')))
    ok = st['state'] == 'done' and not any(st.get('rcs', [1]))
    print('%s: %d traces, %d coverage snapshots in %.0fs' % ('success' if ok else 'failure (%s)' % st['state'], n, ncov, time.time() - t0))
    return 'success' if ok else 'failure', n, logs(jid)


# ------------------------------------------------------------ client: commands

def cmd_tier4(a):
    oracle_dir = a.oracle or default_oracle()
    oh, _ = ensure_build('oracle', oracle_dir, ORACLE_PATHS)
    rh, gate = repo_build()
    seeds = json.loads(agent('alloc-seeds', gate, oh, a.shards, a.seed or 0).stdout)
    per = -(-a.games // a.shards)
    cmds = [{'name': 's%d' % s, 'cwd': 'repo',
             'sh': 'python3 tools/tier4.py --games %d --seed %d --jobs %d --chunk %d --no-stop --out $JOB/out/%d' % (
                 per, s, a.jobs, min(per, a.chunk), s)} for s in seeds]
    jid = submit('tier4', cmds, oracle=oh, repo=rh, nice=10,
                 meta={'gate': gate, 'oracle_hash': oh, 'seeds': seeds, 'games': per * len(seeds), 'oracle_dir': oracle_dir})
    print('vbox: tier4 job %s: %d games, seeds %s, engine %s, oracle %s' % (jid, per * len(seeds), seeds, gate, oh), flush=True)
    if a.detach:
        return 0
    st = wait(jid)
    dest = os.path.join(ROOT, 'corpus/tier4/vbox', jid)
    rsync_from('jobs/%s/out/' % jid, dest)
    for l in logs(jid).split('\n'):
        if re.search(r'\[tier4 seed \d+\] (GREEN|RED)|ORACLE |DIVERGED ', l):
            print(l)
    print('vbox: results in %s' % os.path.relpath(dest))
    return 0 if st['state'] == 'done' and not any(st['rcs']) else 1


def cmd_regen(a):
    oracle_dir = a.oracle or default_oracle()
    oh, _ = ensure_build('oracle', oracle_dir, ORACLE_PATHS)
    rh, _ = repo_build()
    cmds = [{'name': 'shard%02d' % i, 'cwd': 'repo',
             'sh': 'python3 tools/regen_shard.py $JOB/in/headers.json.gz %d %d $JOB/out --jobs %d' % (i, a.shards, a.jobs)}
            for i in range(a.shards)]
    jid = submit('regen', cmds, oracle=oh, repo=rh, nice=10, inputs={'headers.json.gz': a.headers},
                 meta={'oracle_dir': oracle_dir})
    print('vbox: regen job %s (%d shards, oracle %s)' % (jid, a.shards, oh), flush=True)
    st = wait(jid)
    rsync_from('jobs/%s/out/' % jid, a.out)
    for f in sorted(glob.glob(os.path.join(a.out, 'summary-*.txt'))):
        print(open(f).readline().rstrip())
    return 0 if st['state'] == 'done' and not any(st['rcs']) else 1


def cmd_exec(a):
    if not a.cmd:
        die('exec: give a command after --')
    oh = None
    if not a.no_oracle:
        oh, _ = ensure_build('oracle', a.oracle or default_oracle(), ORACLE_PATHS)
    rh, _ = repo_build()
    sh = a.cmd[0] if len(a.cmd) == 1 else ' '.join(shlex.quote(c) for c in a.cmd)
    jid = submit('exec', [{'name': 'cmd', 'cwd': 'repo', 'sh': sh}], oracle=oh, repo=rh, nice=a.nice)
    print('vbox: job %s' % jid, file=sys.stderr, flush=True)
    st = wait(jid)
    print(logs(jid), end='')
    dest = a.out or os.path.join(ROOT, 'corpus/vbox', jid)
    if ssh('find jobs/%s/out -type f | head -1' % jid).stdout.strip():
        rsync_from('jobs/%s/out/' % jid, dest)
        print('vbox: $JOB/out -> %s' % os.path.relpath(dest), file=sys.stderr)
    return 0 if st['state'] == 'done' and not any(st['rcs']) else 1


def cmd_ledger(a):
    rows = json.loads(agent('ledger').stdout)
    _, _, trees = snapshot(ROOT, REPO_PATHS)
    gate = '-'.join(trees[t][:9] for t in GATE_TREES)
    oh = snapshot(a.oracle or default_oracle(), ORACLE_PATHS)[0]
    pairs = {}
    for r in rows:
        p = pairs.setdefault((r['gate'], r['oracle']), {'games': 0, 'failures': 0, 'jobs': set(), 'red': set()})
        p['games'] += r['games']
        p['failures'] += r['failures']
        p['jobs'].add(r['job'])
        if r['failures']:
            p['red'].add(r['job'])
    for (g, o), p in sorted(pairs.items(), key=lambda kv: -kv[1]['games'])[:8]:
        cur = ' <- current' if (g, o) == (gate, oh) else ''
        print('engine %s oracle %s: %d games in %d jobs, %d failures%s' % (g, o, p['games'], len(p['jobs']), p['failures'], cur))
        for j in sorted(p['red'])[:5]:
            print('  red: %s' % j)
    cur = pairs.get((gate, oh))
    if not cur:
        print('current code (engine %s, oracle %s): no finished tier4 jobs yet' % (gate, oh))
        return 1
    green = cur['failures'] == 0 and cur['games'] >= a.gate
    print('GATE %s: %d / %d games on the current code, %d failures' % ('GREEN' if green else 'open', cur['games'], a.gate, cur['failures']))
    return 0 if green else 1


def client(argv):
    ap = argparse.ArgumentParser(prog='vbox.py', description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sp = ap.add_subparsers(dest='command', required=True)
    sp.add_parser('up')
    p = sp.add_parser('stop')
    p.add_argument('--force', action='store_true')
    sp.add_parser('ssh')
    sp.add_parser('setup')
    sp.add_parser('gc')
    p = sp.add_parser('oracle')
    p.add_argument('spec')
    p.add_argument('out')
    p.add_argument('--start', type=int, default=0)
    p.add_argument('--count', type=int, default=64)
    p.add_argument('--procs', type=int)
    p.add_argument('--cov-files', default='')
    p.add_argument('--oracle')
    p = sp.add_parser('tier4')
    p.add_argument('--games', type=int, default=16000)
    p.add_argument('--shards', type=int, default=16)
    p.add_argument('--jobs', type=int, default=4, help='oracle processes per shard')
    p.add_argument('--chunk', type=int, default=250)
    p.add_argument('--seed', type=int, help='first seed block to try (default: random)')
    p.add_argument('--oracle')
    p.add_argument('--detach', action='store_true')
    p = sp.add_parser('regen')
    p.add_argument('headers')
    p.add_argument('out')
    p.add_argument('--shards', type=int, default=16)
    p.add_argument('--jobs', type=int, default=4)
    p.add_argument('--oracle')
    sp.add_parser('exec', help='exec [--out DIR] [--oracle DIR] [--no-oracle] [--nice N] -- CMD ...')
    p = sp.add_parser('ledger')
    p.add_argument('--gate', type=int, default=100000)
    p.add_argument('--oracle')
    p = sp.add_parser('status')
    p.add_argument('job', nargs='?')
    p = sp.add_parser('logs')
    p.add_argument('job')
    p.add_argument('--tail', type=int, default=0)
    p = sp.add_parser('wait')
    p.add_argument('job')
    p = sp.add_parser('fetch')
    p.add_argument('job')
    p.add_argument('dest')
    p = sp.add_parser('cancel')
    p.add_argument('job')
    if argv[:1] == ['exec']:
        return cmd_exec(_exec_args(argv))
    a = ap.parse_args(argv)
    if a.command == 'up':
        print(ensure_up())
    elif a.command == 'stop':
        state, _ = box_state()
        if state == 'running' and not a.force:
            running = [j for j in json.loads(agent('list').stdout) if j['state'] == 'running']
            if running:
                die('jobs still running (%s); --force to stop anyway' % ', '.join(j['id'] for j in running))
        aws('ec2', 'stop-instances', '--instance-ids', INSTANCE)
        print('stopping')
    elif a.command == 'ssh':
        os.execvp('ssh', ['ssh', *SSH_OPTS, 'ubuntu@' + ensure_up()])
    elif a.command == 'setup':
        rsync_to([os.path.join(ROOT, 'tools/vbox_setup.sh')], 'vbox/vbox_setup.sh')
        return ssh('bash vbox/vbox_setup.sh', capture=False).returncode
    elif a.command == 'gc':
        print(agent('gc').stdout, end='')
    elif a.command == 'oracle':
        st, n, log = run_oracle(a.spec, a.out, a.start, a.count, a.cov_files, a.oracle, a.procs)
        for l in [l for l in log.split('\n') if 'status=error' in l or 'status=stuck' in l or 'crashed' in l][:10]:
            print('ORACLE:', l)
        return 0 if st == 'success' else 1
    elif a.command == 'tier4':
        return cmd_tier4(a)
    elif a.command == 'regen':
        return cmd_regen(a)
    elif a.command == 'ledger':
        return cmd_ledger(a)
    elif a.command == 'status':
        if a.job:
            print(json.dumps(status(a.job), indent=1))
        else:
            for j in json.loads(agent('list').stdout)[-20:]:
                print('%-40s %-9s %s' % (j['id'], j['state'], j.get('rcs', '')))
    elif a.command == 'logs':
        print(logs(a.job, a.tail), end='')
    elif a.command == 'wait':
        st = wait(a.job)
        print(json.dumps(st, indent=1))
        return 0 if st['state'] == 'done' and not any(st['rcs']) else 1
    elif a.command == 'fetch':
        rsync_from('jobs/%s/out/' % a.job, a.dest)
    elif a.command == 'cancel':
        print(agent('cancel', a.job).stdout, end='')
    return 0


def _exec_args(argv):
    """`exec [flags] -- CMD...`: argparse REMAINDER mixes flags and command, so split at '--'."""
    rest = argv[1:]
    if '--' in rest:
        flags, cmd = rest[:rest.index('--')], rest[rest.index('--') + 1:]
    else:
        flags, cmd = [], rest
    p = argparse.ArgumentParser(prog='vbox.py exec')
    p.add_argument('--out')
    p.add_argument('--oracle')
    p.add_argument('--no-oracle', action='store_true')
    p.add_argument('--nice', type=int, default=0)
    a = p.parse_args(flags)
    a.cmd = cmd
    return a


# ------------------------------------------------------------- agent (on box)

HOME = '/home/ubuntu'
JOBS, BUILDS, STAGE, NM, VB = (os.path.join(HOME, d) for d in ('jobs', 'builds', 'stage', 'nm', 'vbox'))
BOX_PATH = '/opt/node20/bin:%s/.cargo/bin:/usr/local/bin:/usr/bin:/bin' % HOME


def box_env(extra=None):
    return dict(os.environ, PATH=BOX_PATH, HOME=HOME, **(extra or {}))


def write_json(path, obj):
    tmp = path + '.tmp'
    with open(tmp, 'w') as f:
        json.dump(obj, f, indent=1)
    os.replace(tmp, path)


def alive(pid):
    try:
        os.kill(pid, 0)
        return True
    except (OSError, TypeError):
        return False


def job_status(jid):
    d = os.path.join(JOBS, jid)
    try:
        st = json.load(open(os.path.join(d, 'status.json')))
    except (OSError, ValueError):
        return {'id': jid, 'state': 'unknown'}
    if st['state'] == 'running' and not alive(st.get('pid')):
        st['state'] = 'lost'
    st['id'] = jid
    if st['state'] == 'running':
        prog = []
        for f in sorted(glob.glob(os.path.join(d, 'logs', '*.log'))):
            try:
                with open(f, 'rb') as fh:
                    fh.seek(max(0, os.path.getsize(f) - 2000))
                    lines = [l for l in fh.read().decode('utf-8', 'replace').split('\n') if l.strip()]
                prog.append(lines[-1][:200] if lines else '')
            except OSError:
                pass
        st['progress'] = prog
    return st


def run(cmd, **kw):
    print('$ ' + (cmd if isinstance(cmd, str) else ' '.join(cmd)), flush=True)
    r = subprocess.run(cmd, shell=isinstance(cmd, str), env=box_env(kw.pop('env', None)), **kw)
    if r.returncode:
        sys.exit(r.returncode)


def agent_build(kind, slot, h):
    dest = os.path.join(BUILDS, kind, h)
    stage = os.path.join(STAGE, slot)
    keep = {f for f in sys.stdin.read().split('\0') if f}
    os.makedirs(os.path.join(BUILDS, kind), exist_ok=True)
    with flock(stage + '.lock'):
        if os.path.exists(os.path.join(dest, 'READY')):
            return
        # Drop files deleted locally since the last sync (build outputs stay).
        skip = ('node_modules/', 'output/') if kind == 'oracle' else ()
        for dp, dns, fns in os.walk(stage):
            for fn in fns:
                rel = os.path.relpath(os.path.join(dp, fn), stage)
                if rel not in keep and not rel.startswith(skip):
                    os.remove(os.path.join(dp, fn))
        tmp = '%s.tmp-%s' % (dest, slot)
        shutil.rmtree(tmp, ignore_errors=True)
        os.makedirs(tmp)
        if kind == 'oracle':
            lock = hashlib.sha1(open(os.path.join(stage, 'package-lock.json'), 'rb').read()).hexdigest()[:16]
            nm = os.path.join(NM, lock)
            with flock(nm + '.lock'):
                if not os.path.exists(os.path.join(nm, 'READY')):
                    shutil.rmtree(nm, ignore_errors=True)
                    os.makedirs(nm)
                    for f in ('package.json', 'package-lock.json'):
                        shutil.copy(os.path.join(stage, f), nm)
                    run(['npm', 'ci', '--ignore-scripts', '--no-audit', '--no-fund'], cwd=nm)
                    open(os.path.join(nm, 'READY'), 'w').close()
            link = os.path.join(stage, 'node_modules')
            if os.path.islink(link) or os.path.exists(link):
                os.remove(link) if os.path.islink(link) else shutil.rmtree(link)
            os.symlink(os.path.join(nm, 'node_modules'), link)
            shutil.rmtree(os.path.join(stage, 'output'), ignore_errors=True)
            run(['node', '--max-old-space-size=8192', 'node_modules/typescript/bin/tsc'], cwd=stage)
            shutil.move(os.path.join(stage, 'output'), os.path.join(tmp, 'output'))
            os.symlink(os.path.join(nm, 'node_modules'), os.path.join(tmp, 'node_modules'))
            shutil.copy(os.path.join(stage, 'package.json'), tmp)
        else:
            target = stage + '.target'
            run(['cargo', 'build', '--release', '--bin', 'diff'], cwd=os.path.join(stage, 'engine'),
                env={'CARGO_TARGET_DIR': target})
            run(['rsync', '-a', stage + '/', os.path.join(tmp, 'root') + '/'])
            shutil.copy(os.path.join(target, 'release', 'diff'), os.path.join(tmp, 'diff'))
        # Two worktrees with the same content may build it at once: the first
        # install wins and is never replaced (jobs may already be using it).
        with flock(dest + '.lock'):
            if os.path.exists(os.path.join(dest, 'READY')):
                shutil.rmtree(tmp, ignore_errors=True)
                return
            shutil.rmtree(dest, ignore_errors=True)
            os.rename(tmp, dest)
            open(os.path.join(dest, 'READY'), 'w').close()


def agent_start(jid):
    d = os.path.join(JOBS, jid)
    job = json.loads(sys.stdin.read())
    os.makedirs(os.path.join(d, 'logs'), exist_ok=True)
    for kind in ('oracle', 'repo'):
        if job.get(kind):
            ready = os.path.join(BUILDS, kind, job[kind], 'READY')
            if not os.path.exists(ready):
                sys.exit('%s build %s is missing' % (kind, job[kind]))
            os.utime(ready)   # gc keeps builds in use
    write_json(os.path.join(d, 'job.json'), job)
    write_json(os.path.join(d, 'status.json'), {'state': 'queued'})
    subprocess.Popen([sys.executable, os.path.abspath(__file__), 'agent', 'run', jid], start_new_session=True,
                     stdin=subprocess.DEVNULL, stdout=open(os.path.join(d, 'runner.log'), 'w'), stderr=subprocess.STDOUT)
    print(jid)


def cov_filter(d, files):
    """Keep only the target Twinleaf files' entries of each V8 coverage snapshot (as oracle.yml)."""
    want = tuple('/output/' + f[:-3] + '.js' for f in files)
    out = os.path.join(d, 'out', 'cov')
    os.makedirs(out, exist_ok=True)
    for f in glob.glob(os.path.join(d, 'covraw', '*', 'coverage-*.json')):
        c = json.load(open(f))
        c['result'] = [s for s in c.get('result', []) if s['url'].endswith(want)]
        json.dump(c, open(os.path.join(out, os.path.basename(os.path.dirname(f)) + '-' + os.path.basename(f)), 'w'))
    shutil.rmtree(os.path.join(d, 'covraw'), ignore_errors=True)
    return 0


def agent_run(jid):
    d = os.path.join(JOBS, jid)
    job = json.load(open(os.path.join(d, 'job.json')))
    env = {'JOB': d}
    if job.get('oracle'):
        env['PTCG_ORACLE'] = os.path.join(BUILDS, 'oracle', job['oracle'])
    if job.get('repo'):
        rb = os.path.join(BUILDS, 'repo', job['repo'])
        env.update(PTCG_DIFF=os.path.join(rb, 'diff'), PTCG_DIVERGENCES=os.path.join(rb, 'root', 'divergences.toml'))
    cwds = {'job': d, 'oracle': env.get('PTCG_ORACLE'), 'repo': os.path.join(BUILDS, 'repo', job['repo'] or '', 'root')}
    t0 = time.time()
    procs = []
    for i, c in enumerate(job['cmds']):
        log = open(os.path.join(d, 'logs', '%02d-%s.log' % (i, c['name'])), 'w')
        procs.append(subprocess.Popen(['nice', '-n', str(job.get('nice', 0)), 'bash', '-c', c['sh']], cwd=cwds[c.get('cwd', 'job')],
                                      env=box_env(env), stdout=log, stderr=subprocess.STDOUT, start_new_session=True))
    st = {'state': 'running', 'pid': os.getpid(), 'pgids': [p.pid for p in procs], 'started': t0}
    write_json(os.path.join(d, 'status.json'), st)
    while any(p.poll() is None for p in procs):
        if time.time() - t0 > job.get('timeout', 86400):
            for p in procs:
                if p.poll() is None:
                    with contextlib.suppress(OSError):
                        os.killpg(p.pid, signal.SIGKILL)
            st['timed_out'] = True
        time.sleep(2)
    rcs = [p.returncode for p in procs]
    for c in job.get('post', []):
        if c.get('builtin') == 'cov-filter':
            rcs.append(cov_filter(d, c['files']))
    st.update(state='done', rcs=rcs, finished=time.time())
    write_json(os.path.join(d, 'status.json'), st)


def agent_cancel(jid):
    d = os.path.join(JOBS, jid)
    st = json.load(open(os.path.join(d, 'status.json')))
    for pg in st.get('pgids', []):
        with contextlib.suppress(OSError):
            os.killpg(pg, signal.SIGKILL)
    with contextlib.suppress(OSError, TypeError):
        os.kill(st.get('pid'), signal.SIGKILL)
    st.update(state='cancelled', finished=time.time())
    write_json(os.path.join(d, 'status.json'), st)
    print('cancelled ' + jid)


def jobs_list():
    return [job_status(os.path.basename(p)) for p in sorted(glob.glob(os.path.join(JOBS, '*'))) if os.path.isdir(p)]


def agent_alloc_seeds(gate, oracle, n, first):
    """n tier-4 seed blocks never used for this (engine, oracle) pair: fresh games only."""
    import random
    path = os.path.join(VB, 'seeds.json')
    with flock(path + '.lock'):
        used = json.load(open(path)) if os.path.exists(path) else {}
        key = '%s %s' % (gate, oracle)
        taken = set(used.get(key, []))
        s = first if 1000 <= first < 42000 else random.randrange(1000, 42000)
        out = []
        for _ in range(41000):
            if s not in taken:
                out.append(s)
                if len(out) == n:
                    break
            s = 1000 + (s - 1000 + 1) % 41000
        used[key] = sorted(taken | set(out))
        write_json(path, used)
    print(json.dumps(out))


def agent_ledger():
    """Per tier-4 shard: games and failures (a shard that ended without a summary counts as red)."""
    rows = []
    for st in jobs_list():
        d = os.path.join(JOBS, st['id'])
        try:
            job = json.load(open(os.path.join(d, 'job.json')))
        except (OSError, ValueError):
            continue
        if job['kind'] != 'tier4' or st['state'] in ('queued', 'running', 'cancelled'):
            continue
        for s in job['meta']['seeds']:
            row = {'job': st['id'], 'gate': job['meta']['gate'], 'oracle': job['meta']['oracle_hash'], 'seed': s}
            try:
                sm = json.load(open(os.path.join(d, 'out', str(s), 'summary.json')))
                ok = not sm['failures'] and sm['stats'].get('traces') == sm['games'] and not sm['stats'].get('unsupported')
                row.update(games=sm['games'], failures=len(sm['failures']) or (0 if ok else 1))
            except (OSError, ValueError, KeyError):
                row.update(games=0, failures=1, note='no summary (%s)' % st['state'])
            rows.append(row)
    print(json.dumps(rows))


def agent_gc(quiet=False):
    """Drop job outputs older than 14 days (tier-4 summaries stay for the
    ledger), builds unused for 7 days, and the staging copy and cargo cache of
    every local tree not synced for 14 days."""
    now, freed = time.time(), []
    for st in jobs_list():
        d = os.path.join(JOBS, st['id'])
        if st['state'] in ('queued', 'running') or now - st.get('finished', now) < 14 * 86400:
            continue
        for f in glob.glob(os.path.join(d, 'out', '**', '*'), recursive=True):
            if os.path.isfile(f) and not f.endswith('summary.json'):
                os.remove(f)
        freed.append(st['id'])
    running = {json.load(open(os.path.join(JOBS, st['id'], 'job.json'))).get(k) for st in jobs_list()
               if st['state'] in ('queued', 'running') for k in ('oracle', 'repo')}
    for ready in glob.glob(os.path.join(BUILDS, '*', '*', 'READY')):
        b = os.path.dirname(ready)
        if now - os.path.getmtime(ready) > 7 * 86400 and os.path.basename(b) not in running:
            shutil.rmtree(b, ignore_errors=True)
            freed.append(b)
    for lock in glob.glob(os.path.join(STAGE, '*.lock')):
        slot = lock[:-len('.lock')]
        if now - os.path.getmtime(lock) > 14 * 86400:
            with flock(lock):
                shutil.rmtree(slot, ignore_errors=True)
                shutil.rmtree(slot + '.target', ignore_errors=True)
            os.remove(lock)
            freed.append(slot)
    if not quiet:
        print('\n'.join(freed) or 'nothing to collect')


def agent_idle_check(dry=False):
    """Run by root every 5 minutes (vbox-idle.timer): power off after 30 idle minutes."""
    def stay(why):
        if dry:
            print('stay up: ' + why)
    if any(st['state'] == 'running' for st in jobs_list()):
        return stay('a job is running')
    last = [os.path.getmtime(f) for f in [os.path.join(VB, 'activity')] if os.path.exists(f)]
    last += [st.get('finished', 0) for st in jobs_list()]
    idle = time.time() - max(last or [0])
    if idle < 1800:
        return stay('last activity %.0f min ago' % (idle / 60))
    ss = subprocess.run(['ss', '-Htn', 'state', 'established', '( sport = :22 )'], capture_output=True, text=True).stdout
    if ss.strip():
        return stay('SSH session open')
    if dry:
        return print('would power off')
    agent_gc(quiet=True)
    subprocess.run(['systemctl', 'poweroff'])


def agent_main(argv):
    cmd, args = argv[0], argv[1:]
    if cmd not in ('run', 'idle-check'):
        os.makedirs(VB, exist_ok=True)
        open(os.path.join(VB, 'activity'), 'a').close()
        os.utime(os.path.join(VB, 'activity'))
    if cmd == 'has':
        print('yes' if os.path.exists(os.path.join(BUILDS, args[0], args[1], 'READY')) else 'no')
    elif cmd == 'build':
        agent_build(*args)
    elif cmd == 'start':
        agent_start(args[0])
    elif cmd == 'run':
        agent_run(args[0])
    elif cmd == 'status':
        print(json.dumps(job_status(args[0])))
    elif cmd == 'list':
        print(json.dumps(jobs_list()))
    elif cmd == 'logs':
        tail = int(args[1]) if len(args) > 1 else 0
        for f in sorted(glob.glob(os.path.join(JOBS, args[0], 'logs', '*.log'))):
            lines = open(f, errors='replace').read().split('\n')
            print('==> %s <==' % os.path.basename(f))
            print('\n'.join(lines[-tail - 1:] if tail else lines))
    elif cmd == 'cancel':
        agent_cancel(args[0])
    elif cmd == 'alloc-seeds':
        agent_alloc_seeds(args[0], args[1], int(args[2]), int(args[3]))
    elif cmd == 'ledger':
        agent_ledger()
    elif cmd == 'gc':
        agent_gc()
    elif cmd == 'idle-check':
        agent_idle_check(dry=args[:1] == ['--dry-run'])
    else:
        sys.exit('unknown agent command ' + cmd)


if __name__ == '__main__':
    if len(sys.argv) > 1 and sys.argv[1] == 'agent':
        agent_main(sys.argv[2:])
    else:
        sys.exit(client(sys.argv[1:]))
