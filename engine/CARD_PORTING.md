# Porting a card to the Rust engine

The Rust engine must behave **exactly** like Twinleaf (the fork's `oracle`
branch) on every card: same option sets at every decision, same canonical state
after every step. Where Twinleaf disagrees with the official card text or the
rules, the bug is fixed in both engines (phase 4b, 2026-10-04): see "Fixing a
Twinleaf bug" at the end. Never copy a known bug into Rust.
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
| `MOVE_CARDS(store, state, slot, dest, { cards })` of attached Energy / a whole stack | `move_cards(g, ListRef::Slot(p, s), ...)`. A slot's `energies` is only a view of the Energy cards that are also in its `cards`: moving from `ListRef::SlotEnergies` (Twinleaf: `slot.energies` as the source) leaves the card in `cards` too, so cards must use the slot as the source (Mega Skarmory ex's Sonic Ripper did that wrongly until phase 4b). |
| `SHUFFLE_DECK` | `shuffle_deck(g, p)` |
| `DRAW_CARDS` | `draw_cards(g, p, n)?` |
| `COIN_FLIP_PROMPT(store, state, player, cb)` | `g.coin_flip(p, CoinCb::Card { card: me, frame })?` → the result arrives in your `coin` fn |
| `MULTIPLE_COIN_FLIPS_PROMPT` / `FLIP_UNTIL_TAILS...` | `coin_flip_sequence(g, p, n /*0 = until tails*/, CoinCb::SequenceCard { card: me, frame })?` → `resume` gets `frame.a[2]` = heads bitmask, `frame.a[3]` = flips |
| `IS_ABILITY_BLOCKED(store, state, player, this)` | `is_ability_blocked(g, p, me, None)` |
| `player.switchPokemon(target, store, state)` | `switch_pokemon(g, p, slot)?` (dispatches MovedToActive / MovedFromActiveToBench) |
| `player.switchPokemon(target)` (no `store, state`: most older cards, Kieran, the Catchers) | `switch_pokemon_silent(g, p, slot)?`: same board change, no effects dispatched (matters for ability-lock activation order and MovedToActive handlers). Check the TS call before choosing. |
| `ADD_MARKER / HAS_MARKER / REMOVE_MARKER_AT_END_OF_TURN` | `marker!("NAME")` gives the id; `g.st.players[p].marker.add(...)`, `has_from`, `remove_marker_at_end_of_turn(g, e, m, me)` |
| `ChooseCardsPrompt` on deck/discard | `choose_cards(g, p, "MESSAGE", ListRef::Deck(p), filter, opts, cont)` — it reproduces the constructor's sort of non-secret deck/discard |
| Other prompts | `g.prompt(player_id, "MESSAGE", PromptKind::..., Cont::Card { card: me, frame })` |

The `Class` column of a batch table is not always the logic class: use the
`behavior` field of the card's `CardDef` in `engine/src/gen/cards.rs` (e.g.
`IronThornsexPRE` is `behavior: "IronThornsex"`, so the port is
`iron_thornsex.rs` with `class: "IronThornsex"`).

A lock probe (`is_ability_blocked`) on a card that is still in the hand (a
Pokémon being benched or evolved: PlayPokemon / Evolve handlers run before the
card moves) sees it in a hand list, not a Pokémon slot. Iron Thorns ex's
Initialization treats such a card as in play (since phase 4b), so a Rule Box
Pokémon's on-play Ability (Meowth ex, Durant ex, Archaludon ex, Marnie's
Grimmsnarl ex) is locked while Iron Thorns ex is Active. Other locks may still
return "not locked" for a card in the hand, and for a card in the deck or
discard pile; mirror each lock's own Twinleaf probe.

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
* A coin flip's callback runs after the flip's WaitPrompt is answered, so inside
  a handler for an effect that the core applies right after the card handlers
  (PutDamageEffect, ...) it is too late to change that effect. Cards that need
  the coin before the damage (Annihilape's Durable Body) read
  `CoinFlipEffect.result` instead, which is set at once:
  `g.run_fx(Effect::CoinFlip { callback: None, .. })` (see
  `prefabs::survive_on_ten_on_coin_flip`).
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

Fixed-size arrays in core types can overflow in degenerate stalled games (for
example `EnergyMap = SVec<EnergyEntry, 32>` with 33 Energy attached to one
Pokémon after ~100 turns of Tynamo's Hold Still: `SVec<32> overflow` panic in
`diff`). Scenarios whose Pokémon can only heal should give `me` an attacker.

Keep core edits additive and minimal: other porters are editing the same files
in parallel on other branches.

### New attack effect kinds

Several cards react to *every* effect of an attack (Twinleaf checks
`effect instanceof AbstractAttackEffect` or similar), so their masks list every
kind with an `atk_base`: Mist Energy, Rabsca, Shuppet's `HIDE_N_SNEAK_KINDS`
(fix its array length), Acerola's Mischief, Rock Fighting Energy, Empoleon ex,
Milotic ex and Skeledirge. Acerola's Mischief, Rock Fighting Energy and
Empoleon ex use `HIDE_N_SNEAK_KINDS` itself (an omission there once made
Empoleon ex ignore Pouncing Trap's extra damage). If you add an attack effect
kind, add it to each of those lists, or those cards silently ignore the effect.
Use only the effect kind numbers your batch was given. Effects that act on the
attacker (`SelfPreventRetreat`, `PreventAttackUntilLeavesActive`, ...) must
carry the attacker as `b.target`, or the Defending Pokémon's protections
(Mist Energy, Empoleon ex) would prevent them.

Per-card runtime writes also reach the oracle hash: any Twinleaf write to a
card object's own fields (e.g. `effect.attack.shredAttack = true`) shows up
in the canonical `cards` entry (the whole `attacks` array). Model it on
`CardInst` and emit it in `canonical.rs` (see `attack_shred`). Scenario traces
can't be replayed with `cli.js state` (it ignores the scenario); use a small
node script that passes `scenario: trace.header.scenario` to `GameRunner`.
`canPlay` of Supporters is never reached with `supporterTurn > 0` by the oracle.
The oracle skips `Store.calculatePlayability` (`OracleHooks.noPlayability`):
it ran every `canPlay` of the active player's hand against the live state
after each action, and a probe such as Bianca's Devotion's `CheckHpEffect`
rewrote `hpBonus` of every Pokémon, which Rust (no such probes) never saw.

### Reprints and pins

Reprints are the same card. Twinleaf usually writes a reprint as a subclass
with no logic of its own (`class PalafinexSAR extends Palafinex`), and
`CardDef::behavior` resolves every printing to the class that holds the
logic, so **one unpinned port covers every printing**. Port the logic class
once, unpinned, and test it with whichever printing is in the pool.

Pin a port (`class: "Class@SET"`, `"Class@Full Name"`, `"Class@A|B"`; see
`class_matches` in `engine/src/cards/mod.rs`) only when Twinleaf defines that
class name in more than one file (`grep -rn "export class Judge\b"
twinleaf/ptcg-server/src/sets`): those are different cards that share a name
(`Judge` in FST and SVI). Compare the files: if the logic is the same, widen
the existing pin to both printings; otherwise write a separate port pinned to
the new printing. `python3 tools/check_pins.py` fails on any pin whose class
is defined only once; run it before committing.

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

Card names: every card has two identifiers. The **international key** is the
official English name, set code and printing number from `data/pool.json`
(`key`, e.g. "Growing Grass Energy POR 86", or "Name SET" when unique); use it
in reports, commit messages, `--tag` text people read, and anything else a human
reads. **Twinleaf's `fullName`** ("Grow [G] Energy M3", Japanese set codes and
fan translations) stays the identity inside the oracle, traces, corpus, deck
files and port pins, and `data/verified.json` records both (`key` and
`"twinleaf"`). Tools and `def_by_full_name` accept either form; `tools/names.py`
(`english()`, `twinleaf()`, `label()`) and `carddb::en_name` / `en_key` convert.
When the two differ, write the key first and the Twinleaf name in parentheses.

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
`python3 tools/coverage.py corpus/cards/<tag>/cov <twinleaf file.ts> --min 1000`
prints every segment with its game count (the default only lists those under
3), which is how to paste the line for a branch that is covered. V8 only
reports a range where its count differs from the enclosing one, so a guard's
own `{ return state; }` may not appear as a segment: then the evidence is the
guard's segment at N games with the code after it at 0. Legality trials run
card code too: a `throw` that guards a repeated action (POWER_ALREADY_USED) is
covered by any game where the action stays offered after being used.

A branch is an exemption candidate only when no game with the current pool can
reach it. Common cases:

* Ability-blocked returns when no ported card can lock that Pokémon in that
  position (the pool's locks are narrow: e.g. Team Rocket's Watchtower hits
  [C], Gastrodon hits Benched Stage 2). Iron Thorns ex in the Active Spot
  locks every Rule Box Pokémon (ex, Mega ex, ...) except Future ones, so those
  are reachable: use a scenario with Iron Thorns ex Active (see
  `scenarios/mega-diancie-iron-thorns-lock.json`).
* Empty or null prompt results on prompts that can't be cancelled (min ≥ 1).
* Defensive checks the rules make impossible (an attack that can't be paid for
  without Energy checking for no Energy).

An ability's own `IS_ABILITY_BLOCKED` throw (`BLOCKED_BY_EFFECT`) is not
reachable through a lock that strips the power: `UseAbilityAction` first runs a
`CheckPokemonPowersEffect`, and a remove-mode lock (Gastrodon, Watchtower, Iron
Thorns) removes the power there (`UNKNOWN_POWER`) before the `PowerEffect` ever
exists, so a Chandelure under Gastrodon never reaches its own check. Callbacks
of such locks that return early for a card that is "not in a Pokémon slot" can
be reached with Salvatore (it dispatches `CheckPokemonPowersEffect` for deck
cards); Iron Thorns ex no longer returns early for a card in the hand (Team
Rocket's Arbok and the on-play Abilities probe it there).

Check the actual shape of card-selection effects before exempting a filter or
removal branch. Dedenne SSP's Electromagnetic Sonar can recover any Trainer
from the discard pile, including Neutralization Zone; its explicit selected
cards list reaches Neutralization Zone's stadium-removal filter. Narrower
recovery paths do not make that branch unreachable.

Rare but reachable branches (empty deck, empty opposing hand, 13 cards of a
kind in the discard pile) are not exemptions, and you own them: write a
scenario for each (below). Don't grind random games or `--scout` runs for a
branch that needs a specific board.

### Coverage workflow (required)

1. Parity loop until zero divergences (random games, no coverage).
2. One coverage run of random games (`--coverage`, `--remote` for 32+ games).
3. For every reachable branch still under 3 games, write a scenario that sets
   up the board it needs and run it with `--coverage`. A branch counts as
   covered only when the scenario run's coverage report shows it in ≥3 games,
   with zero divergences. One scenario may cover several branches.
4. Whatever is left is either an exemption candidate (unreachable, with the
   reason) or makes the card partial (say why the scenario didn't reach it).
5. Commit the scenario files on your branch, and list them in your report
   (format below). The orchestrator re-runs them to check the coverage claim.

### Scenarios

A scenario starts every game from a crafted position: the game runs normally
from seed and decks, and at the first turn decision on or after `turn` both
engines apply the same board edits, then play continues and is diffed as
usual. The trace records the hash right after the edits and `diff` checks it,
so a scenario can't silently differ between the engines.

```
python3 tools/check_cards.py "Luxray ex TWM" --scenario scenarios/luxray-ex-empty-hand.json --games 12 --coverage
```

```json
{
  "turn": 2,
  "decks": [["4 Luxray ex TWM", "4 Luxio FST 92", "4 Shinx FST 91", "48 Lightning Energy MEE"], ["..."]],
  "me":  {"active": "Luxray ex TWM", "active_energy": ["3 Lightning Energy MEE"]},
  "opp": {"hand_to_deck": true}
}
```

* `me` is the player whose turn it is at `turn` (default 2, the first turn
  that can attack); `opp` is the other player. A scenario can describe the
  whole board: with `reset` it doesn't matter what turn 1 did.
* Top level: `turn`, `me`, `opp`, `coins` (the next real coin flips, `true` =
  heads; simulation and legality trials are unaffected), `answers` (the next
  decisions, in the trace's answer format, e.g. `{"a": "attack", "name":
  "Piercing Gaze"}`, `{"a": "pass"}`, `{"a": "retreat", "bench": 0}`; prompt
  answers as recorded in traces), `decks`.
* Side edits, in this order:

  | Edit | Effect |
  | --- | --- |
  | `reset: true` | Done for both players before any other edit: every card the player has (hand, discard, Prizes, Stadium, Bench, Active, attachments) goes back to the deck and every slot is emptied. Needs `active`. Prizes not named in `prizes` are refilled from the top of the deck after all other edits. |
  | `hand_to_deck: true` | Whole hand to the bottom of the deck |
  | `discard`, `hand`: `[names]` | Cards moved there |
  | `deck_top`: `[names]` | Cards moved to the top of the deck, first = top (fixes the next draws) |
  | `prizes`: `[names]` | Prize i's card goes to the deck, the named card takes its place |
  | `stadium`: name | Put into play (no Stadium may be in play) |
  | `active`: name or `[Basic, Stage 1, ...]` | Into the Active Spot (benched and switched in, or placed directly after `reset`) |
  | `active_energy`, `active_tool`, `active_damage`, `active_conditions`, `active_played` | Dress the Active Pokémon |
  | `bench`: `[{card, energy, tool, damage, conditions, played}]` | Pokémon (or stacks) on the next empty Bench spots |
  | `supporter_played`, `energy_attached`, `retreated`: `true` | This turn's flags |
  | `prizes_left`: N | Applied last: Prizes N..5 go to the bottom of the deck, so the player has N Prize cards left (e.g. Briar needs the opponent at 2) |

  `conditions`: `PARALYZED`, `CONFUSED`, `ASLEEP`, `POISONED`, `BURNED`.
  `played`: `"earlier"` (default; can evolve, as if in play since an earlier
  turn) or `"this_turn"` (just played: can't evolve yet).
* Cards are taken from the deck (first from the top), else the hand, so the
  decks must contain them. `"4 Name"` repeats a card. English keys work.
* `decks` is optional (default: the usual auto decks). Every deck should hold
  the scenario's cards, since either player may be `me`.
* Decks must be legal: Twinleaf's setup ends a game before it starts when a
  deck fails `DeckAnalyser.isValid` (60 cards, max 4 copies, one ACE SPEC, one
  Radiant, a Basic Pokémon). `check_cards.py` reports those as `INVALID DECK`.
* Effects that last ("during your opponent's next turn...", markers) can't be
  written as edits: set the board, then use `answers` to play the real attack
  or Trainer that creates them.
* Example covering most edits: `scenarios/smoke-full.json`.
* Keep scenarios in `scenarios/`, one file per card and branch:
  `<card-slug>-<branch>.json` (e.g. `luxray-ex-empty-hand.json`). Tag their
  corpora `<batch>-scen-<card-slug>`.
* A scenario must reach the branch through normal play after the edits (the
  policy still picks moves): set the board so the branch is likely, and run
  enough games (12-16 is usually plenty) that it runs in ≥3.

Implementation: `twinleaf/ptcg-server/src/oracle/scenario.ts` and
`engine/src/scenario.rs`, kept identical. Porting agents don't edit them (the
Twinleaf checkout is off limits): if a branch needs an edit that doesn't exist
yet (say, setting a Special Condition or a Prize card), report it as a
scenario request with the branch it would cover, and mark the card partial.

Name every corpus directory with your batch prefix (`--tag bNN-...`) so it can't
collide with other batches. Delete coverage JSONs and `dump/` directories you no
longer need (they are large).

Card status in your report:

Name each card by its international key, with the Twinleaf name in parentheses
when different, e.g. `Growing Grass Energy POR 86 (Grow [G] Energy M3): verified`.

* **verified**: zero divergences, every reachable branch in ≥3 games.
* **partial**: zero divergences, but a reachable branch ran in fewer than 3
  games, or the card was only exercised with a temporary helper.
* **blocked**: the card can't be exercised (missing core feature, oracle
  crash); say exactly what is missing.

Scenarios in your report, one entry per scenario file:

```
scenarios/<file>.json - <International key> (<Twinleaf fullName> if different)
  targets:  <twinleaf file>:<line> `<statement>` (the branch, as the coverage report prints it)
  setup:    <one line: what the edits do and why that reaches the branch>
  result:   <N> traces, 0 diverged; branch ran in <K> games (coverage report line pasted)
  corpus:   corpus/cards/<tag>
```

If a scenario failed to cover its target, list it anyway, with the result and
what you think is missing.

### Notes from batch b17

* `check_cards.py` builds one deck holding every target (up to 4 copies each),
  so more than about 12 distinct targets in one run give `INVALID DECK` games
  that test nothing. Run supporters and tools as separate groups (<= 9 targets).
* Scenario decks: a Special Energy such as Legacy Energy TWM is an ACE SPEC (one
  per deck); use Mist Energy TEF for a plain Special Energy. `answers` cannot name a
  card id (ids depend on the shuffle), so a branch behind "play this Supporter" relies
  on the policy playing it; keep the hand small so it is the only play.
* Coverage counts statements run by legality trials too: a `throw` guard such as
  `CANNOT_PLAY_THIS_CARD` or `SUPPORTER_ALREADY_PLAYED` is reached whenever the card
  is in hand with the guarded board (no need to play it). A second Supporter in the
  hand after the first was played reaches `SUPPORTER_ALREADY_PLAYED`.
* Coin re-flip cards: `ATTACK_COIN_REFLIP_REDUCE_EFFECT` is ported in
  `backtrack_badge.rs` (single flip and sequence). A sequence is wrapped by running
  the default `CoinFlipSequence` with a `CoinCb::SequenceCard` of the card; the
  final step of a sequence is `Game::finish_coin_sequence`.
* "Knocked Out during your opponent's last turn" scenarios: copy
  `scenarios/hassel-ko-previous-turn.json` (scripted `pass`, opponent attack, prize
  and new-Active answers) and put the needed hand in `me.hand`.

## Rules

* Do not modify the Twinleaf checkout. If a card can't be verified because of
  the oracle (e.g. Twinleaf crashes), report it.
* Do not change existing card ports owned by others unless the fix is needed and
  you say so.
* Correct beats faithful: if Twinleaf's behavior differs from the official text
  (`data/official_text.json`), fix Twinleaf and port the fix ("Fixing a Twinleaf
  bug"); if a task forbids touching Twinleaf, match it and report the bug in
  the format below.
* Commit only your batch's ports and the core changes they need. Never commit
  copies of other batches' unmerged ports; if you need one to test, use it
  locally, remove it, and say which corpora depend on it.
* Don't edit `data/verified.json`; report statuses (by international key, Twinleaf name in parentheses when different) and the merger records them.
* Commits carry the configured git identity only: no Co-Authored-By or other
  trailers. Don't push.

### Reporting Twinleaf bugs

One line per bug, so they can be collected into the fix list:

```
<International key> [(<Twinleaf fullName> if different)] (<twinleaf file>:<line>) - <what Twinleaf does> vs <what the card says>
```

For example (this one was fixed in phase 4b): `Team Rocket's Zapdos DRI 70
(team-rockets-zapdos.ts:65) - checks the name 'Team Rocket Energy', so the +60
never applies vs "Team Rocket's Energy"`. Include crashes and stuck prompts (no
valid answer) the same way.

## Fixing a Twinleaf bug (phase 4b)

Card bugs are fixed in the oracle itself (fork christian-shin/twinleafgg,
branch `oracle`) and the same fix is ported to Rust, so parity stays exact
and both engines play the card as printed. The reference is the official text
(`data/official_text.json`, keyed like `data/pool.json`) plus the rulebook.
The tracked list is `porting/twinleaf-fixes.md`.

1. **Twinleaf.** Fix tasks get their own Twinleaf worktree and branch (never
   edit the main `twinleaf/` checkout). Make the smallest change that matches
   the card text, in Twinleaf's style, one commit per card ("Fix <key>:
   <what>"). Build with
   `node --max-old-space-size=4096 ./node_modules/typescript/bin/tsc` in
   `ptcg-server/` (~2 minutes) and run the oracle tools against it with
   `PTCG_ORACLE=<worktree>/ptcg-server` (and `PTCG_ORACLE_REF=<branch>` for
   `--remote`, once the branch is pushed).
2. **Unanswerable prompts.** A card must never open a prompt with no valid
   answer. Follow the card text: when the text says "you can't use/play this
   if ...", or when the card would do nothing at all, make it unplayable the
   way similar Twinleaf cards do (a `throw new GameError(...)` before any
   state change, so the legality trial removes the option); otherwise clamp
   the prompt's `min` to what is available. Crashes inside a prompt callback
   are always bugs. A guard that runs after a prompt (a throw inside a
   callback, e.g. "accepting the ability with no target") must move before
   it, so the ability isn't offered. Legality trials draw fixed outcomes
   (every coin tails), and a Trainer played under Seismitoad's Quaking Fist
   flips a coin first: the trial therefore skips that flip and checks the
   heads path (`Chance.inTrial` in the oracle, `Rng::is_fixed` in Rust),
   otherwise an unplayable card looks legal and throws on real heads.
3. **Rust.** Port the same change. If the fix is in a core file (a prompt's
   `validate`, a prefab), port it in the matching core file and list every
   card that goes through it.
4. **Regression scenario** `scenarios/fix-<card-slug>-<branch>.json` that runs
   the fixed branch in >=3 games (`--coverage`), zero divergences. Then
   re-run the card with random games and every existing scenario that names it
   (`grep -l "<name>" scenarios/*.json`), all zero divergences.
5. **Corpus.** Replay the main corpora (`diff corpus/t1 corpus/meta1
   corpus/meta2 corpus/cards/*/ --quiet`). Traces recorded under the buggy
   oracle may now diverge; that is expected only when the trace actually hit
   the bug. Check each diverged trace's first divergence and list it in the
   report with its cause. Don't regenerate or delete them; the orchestrator
   rebuilds corpora after all fixes merge.
6. Never add a card bug to `divergences.toml`.

7. **Legality trials** (`legalTurnOptions` / `options::is_legal`) dispatch each
   candidate action and stop at the first decision or chance prompt, so a
   throw that comes after such a prompt never reaches the trial. The Confusion
   flip is the exception: the trial resolves it as heads, so a Confused
   attacker is not offered an attack that throws once the flip succeeds (the
   game used to end with `status: error`). For an Ability that picks an attack
   or a Supporter and then runs it (Mew ex's Memory Helix, Mr. Mime's
   Look-Alike Show), do not rely on the trial: block the unusable choices in
   the prompt (`blocked`), and when the chosen one throws anyway, restore the
   game phase and prompt again without it (see `copy_attack::prompt_ability`).

Report per fix: list number, card (international key), Twinleaf commit,
Rust change, scenario entry (format above), the official text it now
follows, and the diverged traces with their causes.
