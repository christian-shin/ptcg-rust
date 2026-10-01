# Porting a card to the Rust engine

The Rust engine must behave **exactly** like Twinleaf (commit `41382b8`, oracle
branch) on every card: same option sets at every decision, same canonical state
after every step. Twinleaf bugs are part of the spec; port them faithfully.
A card is done when `tools/check_cards.py` reports zero divergences over traces
that exercise every reachable branch of its code in at least 3 games.

## Where things are

| What | Where |
| --- | --- |
| Twinleaf source of a pool card | `data/pool.json` → `twinleaf_file` (relative to `twinleaf/ptcg-server/src`); the oracle checkout lives at `/Users/christianshin/Documents/pkmntcg/twinleaf/ptcg-server` |
| Twinleaf core / prefabs | `twinleaf/ptcg-server/src/game/store/{prefabs,effects,effect-reducers,reducers,prompts}` |
| Card behavior class | `CardDef::behavior` in `engine/src/gen/cards.rs` (reprints share the base class) |
| Rust card ports | `engine/src/cards/impls/<snake_case_class>.rs`, one file per behavior class; registered automatically by `build.rs` |
| Shared helpers (prefab ports) | `engine/src/prefabs.rs` |
| Core rules | `engine/src/engine/*.rs`, store in `engine/src/game.rs` |
| Effects | `engine/src/effects.rs` (one variant per Twinleaf effect class) |
| Prompts | `engine/src/prompts.rs` |
| State | `engine/src/state.rs`; canonical JSON in `engine/src/canonical.rs` |

## Anatomy of a port

```rust
//! Night Stretcher (SFA): ...card text...
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "NightlyStretcher",          // exact Twinleaf class name (CardDef::behavior);
                                        // `Class@SET` / `Class@Full Name` pins it to one printing,
                                        // `Class@A|B` to several (same class name, different files)
    mask: mask(&[k::TRAINER]),          // every Effect kind the TS reduceEffect reacts to
    reduce,                             // TS reduceEffect
    resume: Some(resume),               // continuations (prompt callbacks, generator resumes)
    coin: None,                         // coin flip callbacks
    can_play: None,                     // unused by the rules (UI playability only)
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R { ... }
fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R { ... }
```

`me` is this card instance; `e` indexes the effect being propagated
(`g.e(e)` / `g.e_mut(e)`). The mask must include every effect kind the
Twinleaf `reduceEffect` handles, or the card silently never sees it.

### Translating TypeScript idioms

| Twinleaf | Rust |
| --- | --- |
| `effect instanceof AttackEffect && effect.attack === this.attacks[i]` / `WAS_ATTACK_USED(effect, i, this)` | `was_attack_used(g, e, i, me)` then `match *g.e(e) { Effect::Attack { p, opp, damage, source, .. } => ... }` |
| `WAS_POWER_USED(effect, i, this)` | `was_power_used(g, e, i, me)` (never matches the lock probe) |
| `effect instanceof TrainerEffect && effect.trainerCard === this` | `if let Some(p) = trainer_played(g, e, me)` |
| `effect.damage += 30` | `if let Effect::Attack { damage, .. } = g.e_mut(e) { *damage += 30 }` |
| `effect.preventDefault = true` | `g.set_prevent(e, true)` |
| `throw new GameError(GameMessage.X)` | `bail!("X")` |
| `store.reduceEffect(state, new XEffect(...))` | `g.run_fx(Effect::X { .. })?` → returns `(final effect, prevented)` |
| `StateUtils.getOpponent(state, player)` | `1 - p` (players are indices 0/1; `g.player_id(p)` gives the Twinleaf id for prompts) |
| `player.active` / `player.bench[i]` | `g.st.players[p].active` / `.bench.as_slice()[i]` — **slot ids** in an arena; `SlotRef::new(p, slot)` addresses one |
| `cardList.getPokemonCard()` | `g.st.slot_pokemon(p, slot)` |
| `MOVE_CARDS(store, state, from, to, { cards })` | `move_cards(g, from, to, &cards, me)?` (ListRef::Hand(p), Deck, Discard, Slot(p, s), ...) |
| `player.hand.moveCardTo(c, dest)` (direct, no effect) | `g.move_card_to(ListRef::Hand(p), c, dest)` |
| `SHUFFLE_DECK` | `shuffle_deck(g, p)` |
| `DRAW_CARDS` | `draw_cards(g, p, n)?` |
| `COIN_FLIP_PROMPT(store, state, player, cb)` | `g.coin_flip(p, CoinCb::Card { card: me, frame })?` → the result arrives in your `coin` fn |
| `MULTIPLE_COIN_FLIPS_PROMPT` / `FLIP_UNTIL_TAILS...` | `coin_flip_sequence(g, p, n /*0 = until tails*/, CoinCb::SequenceCard { card: me, frame })?` → `resume` gets `frame.a[2]` = heads bitmask, `frame.a[3]` = flips |
| `IS_ABILITY_BLOCKED(store, state, player, this)` | `is_ability_blocked(g, p, me, None)` |
| `ADD_MARKER / HAS_MARKER / REMOVE_MARKER_AT_END_OF_TURN` | `marker!("NAME")` gives the id; `g.st.players[p].marker.add(...)`, `has_from`, `remove_marker_at_end_of_turn(g, e, m, me)` |
| `ChooseCardsPrompt` on deck/discard | `choose_cards(g, p, "MESSAGE", ListRef::Deck(p), filter, opts, cont)` — it reproduces the constructor's sort of non-secret deck/discard |
| Other prompts | `g.prompt(player_id, "MESSAGE", PromptKind::..., Cont::Card { card: me, frame })` |

### Continuations: generators and callbacks

Twinleaf cards suspend with `yield store.prompt(state, prompt, result => { ...; next(); })`.
Port a generator as stages: the code before the first yield runs in `reduce`;
each `yield` becomes `g.prompt(..., Cont::Card { card: me, frame: CardFrame { stage: N, .. } })`
followed by `return Ok(())`; `resume` matches on `f.stage` and continues. Keep
the locals a later stage needs in `frame.a` (i32s), `frame.e` (effect ids),
`frame.l` (u8s).

Order matters exactly as in TypeScript:

* A prompt callback runs when the prompt is answered; code after a
  non-yielding `store.prompt(...)` call runs **immediately** (before the answer).
* `if (store.hasPrompts()) yield store.waitPrompt(state, () => next())` is
  `if g.has_prompts() { g.wait_prompt(Cont::Card { .. }); return Ok(()) }`.
  Wait items run when no prompt is pending, **last in first out**.
* Info prompts (`ShowCardsPrompt`, `AlertPrompt`, `ConfirmCardsPrompt`,
  `WaitPrompt`) are real prompts: create them where Twinleaf does.
* If a callback uses the effect object after the prompt resolves (for example,
  it adds damage to the `AttackEffect`), `g.retain_fx(e)` before prompting, store
  `e` in the frame, and `g.release_fx(e)` when done.
* A callback that would throw in TypeScript (e.g. `selected[0]` on a cancelled
  prompt) must `bail!` too.

### State Twinleaf mutates that Rust doesn't model yet

Many `PokemonCardList` / `Player` fields exist in Twinleaf but aren't in Rust
because no ported card set them yet (look for "not modeled" comments in
`engine/src/engine/*.rs`). When your card sets such a field:

1. Add it to `Slot` / `Player` in `state.rs` with the TS default.
2. Emit it in `canonical.rs` under the **exact TS field name** when it differs
   from the default.
3. Port every core code path that reads, clears or rolls it over (end of turn in
   `phase.rs`, `clear_effects` / `remove_attack_effects` / `reset_empty_slot` in
   `game_effect.rs`, damage code in `attack.rs`, ...), following the TS source.

Same for new effect classes (`effects.rs`: variant + `type_name` + `kind` + a
`k::` constant) and prompt kinds (`prompts.rs`: kind, descriptor, decode/validate
matching `oracle/options.ts` `describePrompt` and the TS prompt's `validate`).

Keep core edits additive and minimal: other porters are editing the same files
in parallel on other branches.

### New attack effect kinds

Several cards react to *every* effect of an attack (Twinleaf checks
`effect instanceof AbstractAttackEffect` or similar), so their masks list every
kind with an `atk_base`: Mist Energy, Rabsca, Shuppet's `HIDE_N_SNEAK_KINDS`
(fix its array length), Acerola's Mischief, Rock Fighting Energy and Empoleon
ex. If you add an attack effect kind, add it to each of those lists, or those
cards silently ignore your attack. Use only the effect kind numbers your batch
was given.

### Same class name, different file

Twinleaf sometimes has two classes with the same name in different files
(`Judge` in FST and SVI). A port binds every printing whose behavior class has
its name unless pinned: `class: "Class@SET"`, `"Class@Full Name"`, or several
with `"Class@A|B"` (`class_matches` in `engine/src/cards/mod.rs`). Compare the
two source files: if the logic is the same, widen the existing pin; otherwise
write a separate port pinned to the new printing.

### Pre-evolutions outside the pool

Pool Pokémon often evolve from cards that aren't pool cards (Shinx and Luxio
for Luxray ex). These are support cards: `tools/support_cards.py` picks one
printing per missing name (preferring one with no card logic) into
`data/support_cards.json`, and `tools/gen_carddb.py` appends them to the card
DB. Don't generate a temporary card DB to get a pre-evolution into play; if a
needed one is missing, add it there (re-run both scripts) and say so. Support
cards with card logic need a port like any other card.

## Verify

```
cd engine && cargo build --profile iter --bins     # seconds per rebuild; `cargo test --release` before committing
python3 tools/check_cards.py "Full Name A" "Full Name B"            # parity loop: 16 games, no coverage
python3 tools/check_cards.py "Full Name A" "Full Name B" --coverage  # once, at the end
```

More games, faster: add `--remote 8` (up to 20) to run the oracle games on
GitHub Actions runners instead of locally (`.github/workflows/oracle.yml`,
driven by `tools/remote_oracle.py`; needs `gh` logged in). Traces and coverage
come back into the same corpus directory; the Rust diff still runs locally.
Expect ~1-2 minutes of queue/setup overhead, so use it for runs of 32+ games.

New worktree? Copy a warm build cache first so the first build isn't from scratch:
`cp -Rc /Users/christianshin/Documents/pkmntcg/engine/target/iter engine/target/` (APFS clone, instant).

Card names: tools and `def_by_full_name` accept the official English key from
`data/pool.json` (`key`, e.g. "Grand Tree SCR 136", or "Grand Tree SCR" when
unique) as well as Twinleaf's `fullName` ("Great Tree SCR"), which stays the
identity in traces, the oracle and port pins. `carddb::en_name` / `en_key`
give the English name of a card.

`check_cards.py` builds decks around the targets (Stage 1/2 targets need their
pre-evolution in the target list, already ported, or a support card), generates oracle traces
(policies `heur` and `random`, plus one light bot mix), and replays them through
Rust with the newest `diff` build (`iter` or `release`). Jobs default to half
the cores (`PTCG_JOBS` overrides). On a divergence:

* `DIVERGED <trace> at step N: hash|options|prompt|error` — the Rust state at
  that step is in `corpus/cards/<tag>/dump/`.
* Oracle state at that step:
  `cd /Users/christianshin/Documents/pkmntcg/twinleaf/ptcg-server && node output/oracle/cli.js state <trace.json> N > /tmp/o.json`
  (step `-1` = start). It also prints the effect types (`e`) of that step.
* `python3 tools/statediff.py /tmp/o.json <dump>.rust.json` lists differing paths.
* Also re-run the previous corpora to catch regressions:
  `engine/target/iter/diff corpus/t1 corpus/cards/*/ --quiet`. `diff` does not
  recurse: `corpus/cards` alone checks nothing, so pass the subdirectories.

Coverage: a pass means nothing unless the card's branches ran. `--coverage`
records V8 block coverage per game (1.6x slower oracle, ~11 MB per game, so
only for the final run) and lists every block of the card's Twinleaf code that
ran in fewer than 3 games (`--min-games`). Module setup, class declarations and
constructors (once per process) are excluded automatically, and `|| []` /
`?? []` prompt-result fallbacks are counted as exempt. Whatever is still listed
is a real gap: add seeds (`--seed`) or a custom spec, or hunt it with
`--scout 2000` (plays 2000 candidate games in Rust and replays only the most
varied target-heavy ones in the oracle). Coverage line numbers can refer to
the compiled JavaScript rather than the TypeScript source; match the reported
statement, not just the number.

A branch is an exemption candidate only when no game with the current pool can
reach it. Common cases:

* Ability-blocked returns when no ported card can lock that Pokémon in that
  position (the pool's locks are narrow: e.g. Team Rocket's Watchtower hits
  [C], Gastrodon hits Benched Stage 2).
* Empty or null prompt results on prompts that can't be cancelled (min ≥ 1).
* Defensive checks the rules make impossible (an attack that can't be paid for
  without Energy checking for no Energy).

Rare but reachable branches (empty deck, empty opposing hand) are not
exemptions: hunt them with seeds, custom decks or `--scout`, and if they still
don't run, report the card as partial.

Name every corpus directory with your batch prefix (`--tag bNN-...`) so it can't
collide with other batches. Delete coverage JSONs and `dump/` directories you no
longer need (they are large).

Card status in your report:

* **verified**: zero divergences, every reachable branch in ≥3 games.
* **partial**: zero divergences, but a reachable branch ran in fewer than 3
  games, or the card was only exercised with a temporary helper.
* **blocked**: the card can't be exercised (missing core feature, oracle
  crash); say exactly what is missing.

## Rules

* Do not modify the Twinleaf checkout. If a card can't be verified because of
  the oracle (e.g. Twinleaf crashes), report it.
* Do not change existing card ports owned by others unless the fix is needed and
  you say so.
* Faithful beats correct: if Twinleaf's behavior differs from the printed text,
  match Twinleaf and report it (format below). These bugs are later fixed in
  both engines, so precise reports matter.
* Commit only your batch's ports and the core changes they need. Never commit
  copies of other batches' unmerged ports; if you need one to test, use it
  locally, remove it, and say which corpora depend on it.
* Don't edit `data/verified.json`; report statuses and the merger records them.
* Commits carry the configured git identity only: no Co-Authored-By or other
  trailers. Don't push.

### Reporting Twinleaf bugs

One line per bug, so they can be collected into the fix list:

```
<Full Name> (<twinleaf file>:<line>) - <what Twinleaf does> vs <what the card says>
```

For example: `Team Rocket's Zapdos DRI (team-rockets-zapdos.ts:65) - checks
the name 'Team Rocket Energy', so the +60 never applies vs "Team Rocket's
Energy"`. Include crashes and stuck prompts (no valid answer) the same way.
