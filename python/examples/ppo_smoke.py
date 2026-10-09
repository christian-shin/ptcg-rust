"""PPO smoke run against the bindings (Python access).

A numpy-only pointer policy: every legal option gets a score from a linear
function of its features (option one-hots crossed with a few board scalars);
the policy is a softmax over the option list. The learner plays seat 0 of a
VecEnv against a uniformly random opponent. The run passes if the win rate
over the last iterations is clearly above the first.

    .venv/bin/python python/examples/ppo_smoke.py [iterations]
"""
import os
import sys
import time

import numpy as np

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))
import ptcg  # noqa: E402

ROOT = os.path.join(os.path.dirname(__file__), "..", "..")
K = 32
N_ENVS = 64
ROLLOUT = 64

# One-hot widths for option fields: type, area, inPlayArea, selectType, context.
WIDTHS = [(0, 18), (2, 14), (5, 14), (10, 12), (11, 52)]
BASE = sum(w for _, w in WIDTHS)
BOARD = 6  # scalars crossed with each option one-hot


def board_scalars(obs):
    g = ptcg.OBS_SIZE
    s = np.zeros((obs.shape[0], BOARD), np.float32)
    s[:, 0] = 1.0
    s[:, 1] = obs[:, 0] / 20.0            # turn
    s[:, 2] = obs[:, 5]                   # energy attached this turn
    p0 = 16
    s[:, 3] = obs[:, p0 + 3] / 6.0        # own prizes left
    p1 = 16 + (g - 16 - 20) // 2
    s[:, 4] = obs[:, p1 + 3] / 6.0        # opponent prizes left
    s[:, 5] = obs[:, p0 + 16 + 2]          # own active damage
    return s


def features(obs, opts):
    n = opts.shape[0]
    oh = np.zeros((n, K, BASE), np.float32)
    off = 0
    for col, w in WIDTHS:
        idx = np.clip(opts[:, :, col] + 1, 0, w - 1)
        np.put_along_axis(oh[:, :, off:off + w], idx[:, :, None], 1.0, axis=2)
        off += w
    b = board_scalars(obs)
    return (oh[:, :, :, None] * b[:, None, None, :]).reshape(n, K, BASE * BOARD)


def policy(theta, feats, counts):
    logits = feats @ theta
    mask = np.arange(K)[None, :] < np.maximum(counts, 1)[:, None]
    logits = np.where(mask, logits, -1e9)
    logits -= logits.max(axis=1, keepdims=True)
    p = np.exp(logits) * mask
    p /= p.sum(axis=1, keepdims=True)
    return p


def main():
    iters = int(sys.argv[1]) if len(sys.argv) > 1 else 40
    deck_a = ptcg.read_deck(os.path.join(ROOT, "decks/t1-grass.txt"))
    deck_b = ptcg.read_deck(os.path.join(ROOT, "decks/t1-grass.txt"))  # mirror: random baseline 0.5
    env = ptcg.VecEnv(N_ENVS, deck_a, deck_b, seed=1)
    rng = np.random.default_rng(0)
    dim = BASE * BOARD
    theta = np.zeros(dim, np.float32)
    v_w = np.zeros(BOARD, np.float32)
    lr, clip, gamma = 0.5, 0.2, 0.995
    history = []
    t0 = time.time()
    for it in range(iters):
        buf = []
        wins = losses = 0
        for _ in range(ROLLOUT):
            obs, opts, counts = env.observe(K)
            f = features(obs, opts)
            p = policy(theta, f, counts)
            a = np.array([rng.choice(K, p=p[i]) for i in range(N_ENVS)])
            r, d = env.step(a)
            wins += int((r > 0).sum())
            losses += int((r < 0).sum())
            buf.append((f, counts, a, p[np.arange(N_ENVS), a], board_scalars(obs), r, d))
        # Returns with episode boundaries, value baseline.
        ret = np.zeros(N_ENVS, np.float32)
        rets = []
        for (_, _, _, _, _, r, d) in reversed(buf):
            ret = r + gamma * ret * (~d)
            rets.append(ret.copy())
        rets.reverse()
        allv = np.concatenate([(bs @ v_w) for (_, _, _, _, bs, _, _) in buf])
        alla = np.concatenate(rets) - allv
        a_mean, a_std = alla.mean(), alla.std() + 1e-6
        for epoch in range(4):
            g_theta = np.zeros_like(theta)
            g_v = np.zeros_like(v_w)
            for (f, counts, a, p_old, bs, _, _), R in zip(buf, rets):
                v = bs @ v_w
                adv = (R - v - a_mean) / a_std
                v_adv = R - v
                p = policy(theta, f, counts)
                pa = p[np.arange(N_ENVS), a]
                ratio = pa / np.maximum(p_old, 1e-8)
                use = (np.abs(ratio - 1) < clip) | (np.sign(ratio - 1) != np.sign(adv))
                w = (adv * ratio * use)[:, None]
                # d log p_a / d theta = f_a - sum_k p_k f_k
                fa = f[np.arange(N_ENVS), a]
                ef = (p[:, :, None] * f).sum(axis=1)
                g_theta += (w * (fa - ef)).mean(axis=0)
                g_v += (v_adv[:, None] * bs).mean(axis=0)
            theta += lr * g_theta / len(buf)
            v_w += 0.1 * g_v / len(buf)
        games = wins + losses
        rate = wins / games if games else float("nan")
        history.append((games, rate))
        print(f"iter {it:3d}  games {games:4d}  win rate {rate:.3f}  ({time.time() - t0:.0f}s)")
    first = [r for g, r in history[:5] if g]
    last = [r for g, r in history[-5:] if g]
    print(f"win rate first 5 iters {np.mean(first):.3f} -> last 5 iters {np.mean(last):.3f}")


if __name__ == "__main__":
    main()
