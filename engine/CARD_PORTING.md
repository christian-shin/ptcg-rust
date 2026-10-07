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
| `player.switchPokemon(target)` (no `store, state`: most older cards, the Catchers) | `switch_pokemon_silent(g, p, slot)?`: same board change, no effects dispatched (matters for ability-lock activation order and MovedToActive / MovedFromActiveToBench handlers such as Yanmega ex Buzz Boost and Palafin Zero to Hero). Check the TS call before choosing; the silent form is a bug on a card whose text switches a Pokémon (Kieran, Surfer and Team Rocket's Giovanni were fixed to dispatch in the phase 4b review, R4). |
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

### Energy removed as an effect of an attack (R7A, phase 4b)

Energy that leaves a Pokémon as an effect of an attack (discarded, shuffled into
the deck, put into the hand, moved to another Pokémon) leaves **after the damage**: the player chooses the
Energy first, the damage is done with the Energy still attached (Double Turbo
Energy, Voltaic Lightning Energy, Spiky Energy, Ancient Bulwark), then the
Energy is removed (attack flow chart; rulings 1553, 1580, 1846, 1874). Both
engines keep a window on the attack's `AttackEffect` (`afterDamageEffects` in
`prefabs/after-damage.ts`, `Game::after_dmg` and `fx_flag::AFTER_DMG_OPEN` in
Rust): it opens before the AttackEffect is reduced, `DiscardCardsEffect` /
`CardsToHandEffect` (Energy cards only) and `MoveOpponentEnergyEffect` reduced inside it are queued by
`reduceEffect` / `reduce_effect` and run after the DealDamage step, before
AfterAttackEffect (in `useAttack` and in both copy-attack delegations). A card
that moves Energy with `MOVE_CARDS` passes `afterDamageOf: effect`
(`move_cards_after_damage(g, atk, ...)`) and defers the following shuffle with
`SHUFFLE_DECK_AFTER_DAMAGE` (`shuffle_deck_after_damage`). Cards that discard in
an AfterAttackEffect handler (Larvitar, Zapdos, Scream Tail, ...) are unaffected:
the window is closed there. A cost written in the attack cost line, a discard
from the hand (Hydrapple, Ceruledge PFL) and discards of non-Energy cards are not
deferred. An effect with an empty card list is a probe ("does Mist Energy prevent
this?", Ceruledge SSP, Minccino TEF, Illumise TWM, ...) and is reduced at once,
never queued. Retaliation of an Energy on the Defending Pokémon (Spiky Energy) is
step 7 of the flow chart, after the attack's own effects: it is queued in the
window (`AFTER_DAMAGE_OR_NOW` / `AfterDmgStep::Retaliate`) and only happens when
the card is still attached, so Duraludon PFL's Hyper Beam discarding Spiky Energy
stops it (no ruling names this case; the flow-chart order decides).

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

### Shred ("isn't affected by any effects on your opponent's Active Pokémon")

An attack with that text (Shred, Demolish, Twin Shotels, Azure Wave, ...) sets
`AttackEffect.ignoreDefenderEffects` (`Effect::Attack.ignore_defender_effects`;
`THIS_ATTACKS_DAMAGE_ISNT_AFFECTED_BY_EFFECTS` in Twinleaf, `shred` /
`shred_ex` in `mega_lopunnyex.rs`) and lets the damage go through the normal
DealDamage / PutDamage path; never write the damage straight to the Pokémon.
`ignoresDefenderEffects(effect)` / `prefabs::ignores_defender_effects` is true
for such damage done to the opponent's Pokémon. Rulings 1439, 1345, 1629,
1875, 1490: every effect *on the damaged Pokémon* that changes this damage is
skipped (prevention, reduction, extra damage taken, coin-flip prevention,
Tera/Bench protection, Abilities, Tools, Stadiums). Rulings 1716, 1816, 531,
532, 812, 941: effects on the attacker (Maximum Belt, Binding Mochi, "attacks
used by the Defending Pokémon do N less", ...), Weakness and Resistance (and
effects that change them) still apply. Rulings 936, 1770: "survive on 10 HP"
effects still apply after the full damage. A card hook on `PutDamageEffect`
that changes or prevents the damage of the Pokémon it sits on must start with
`!ignoresDefenderEffects(effect)` (Rust: `ignores_defender_effects(g, &b)`);
hooks on `DealDamageEffect` are attacker-side bonuses and must not. Generic
"prevent all damage and effects" hooks (Milotic ex, Acerola's Mischief) skip the
damage steps only (`isDamageIgnoringDefenderEffects`).

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
driven by `tools/remote_oracle.py`; needs `gh` logged in and the Twinleaf
branch pushed, selected with `PTCG_ORACLE_REF=<branch>`). Traces and coverage
come back into the same corpus directory; the Rust diff still runs locally.
Expect ~1-2 minutes of queue/setup overhead, so use it for runs of 32+ games.
If the main checkout has a local remote runner (`porting/remote_runner.py`,
not in git), `--remote` uses it instead; its notes are under `porting/` there.

New worktree? Copy a warm build cache first so the first build isn't from scratch:
`cp -Rc /Users/christianshin/Documents/pkmntcg/engine/target/iter engine/target/` (APFS clone, instant).

Card names: the Rust engine speaks **official English names** everywhere a
person or tool sees them, and keeps Twinleaf's names hidden for the oracle
boundary (2026-10-07).

| Rust field | Value |
| --- | --- |
| `CardDef.full_name` | the international key `"<Name> <SET> <NUM>"` from `data/pool.json` (`key`, e.g. "Growing Grass Energy POR 86"); cards outside the pool keep Twinleaf's full name |
| `CardDef.name`, `set`, `set_number` | official card name, international set code and printing number |
| `AttackDef.name`, `PowerDef.name` | official attack / Ability name (`data/official_text.json`: "Turbo Flare", "Resolute Heart") |
| `CardDef.tl_full_name`, `tl_name`, `tl_set`, `tl_set_number`; `AttackDef.tl_name`, `PowerDef.tl_name` | Twinleaf's ("Grow [G] Energy M3", "Flame Turbo", "Tenacious Heart") |

`tools/gen_carddb.py` fills both from `data/pool.json` and the official text
(`tools/official_names.py`; never edit `engine/src/gen/cards.rs`). A card that is
not in the pool (an old printing, a support card) inherits the official card,
attack and Ability names of a pool card with the same Twinleaf card name, so
same-name rules behave exactly as before; otherwise it keeps Twinleaf's.

**Which name to use in a port.** Rules wording ("a card named X", "Pokémon
with the same name", an Ability called X) compares `name`; write official names
in literals. Everything that is hashed, described to the oracle or kept in the
state uses the `tl_*` names, so the oracle's hash and traces never change:
`canonical.rs` (attack names, `lastAttack`, card refs `<tl_set>-<tl_number>#id`),
turn-option descriptors (`options.rs`: `{a:'attack', name, from}` carry the
Twinleaf attack / Ability name and the Twinleaf `from` full name), prompt
descriptors (`prompts.rs`, ChooseAttack attacks, filter names through
`tl_card_name`), Select values, Rust `Action::Attack { name, from }`, and
attack names stored in the state (`blocked_attack_name_*`,
`cannot_use_attacks_next_turn*`, `NextTurnAttackDamageBonus`: store and compare
`AttackDef.tl_name`, never `name`; two cards can share an official attack name
that Twinleaf spells differently). Sorting that mirrors the oracle's
(`sort_list`) sorts by `tl_name`. `Class@SET` / `Class@Full Name` pins in a
port's `class:` use Twinleaf's set and full name (`tl_set`, `tl_full_name`).
The RL interface (`interface.rs`) emits official names (`{"index","attack"}`
answers) and accepts either spelling.

Tools and `def_by_full_name` accept the international key, "Name SET" when
unique, or Twinleaf's full name; `tools/names.py` (`english()`, `twinleaf()`,
`label()`, `move_twinleaf()`) and `carddb::en_name` / `en_key` convert.
**Scenario files may use either name** (official preferred): `check_cards.py`
maps card names, scripted-answer attack / Ability names and `from` to Twinleaf's
before the scenario reaches the oracle (whose `scenario.ts` / `runner.ts` are
unchanged), and `expect` / `legal` assertions in the Rust replay match both.
Reports write the key first and the Twinleaf name in parentheses when they differ.

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
  | `supporter_played`, `energy_attached`, `retreated`, `stadium_played`: `true` | This turn's flags |
  | `deck_left`: N | Very last: cards from the top of the deck go to the discard pile until N are left (an empty deck: the owner loses at the beginning of their next turn) |
  | `prizes_left`: N | Applied last: Prizes N..5 go to the bottom of the deck, so the player has N Prize cards left (e.g. Briar needs the opponent at 2) |

  `conditions`: `PARALYZED`, `CONFUSED`, `ASLEEP`, `POISONED`, `BURNED`.
  `played`: `"earlier"` (default; can evolve, as if in play since an earlier
  turn) or `"this_turn"` (just played: can't evolve yet).
* Cards are taken from the deck (first from the top), else the hand, so the
  decks must contain them. `"4 Name"` repeats a card. English keys work.
* Top level `sudden_death: true` marks the game as a Tiebreaker game (a Prize taken then counts for Prize advantage).
* In `answers`, `{"a": "play" | "ability", "card_prefix": "PRE-100#"}` uses any copy of that card (instance ids depend
  on the shuffle); the prefix is `<Twinleaf set>-<number>#`.
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

#### Attacks copied by Memory Helix

Mew ex 30C's Memory Helix is passive: the attacks of its Benched Pokemon are
added to its attack options (`CheckPokemonAttacksEffect.copiedAttacks`, Rust
`copied`) while it is Active and the Ability isn't blocked, and run with
`UseAttackEffect.delegateFrom` = the Benched card. The turn-option descriptor of
a copied attack always carries its source: `{a:'attack', name, from:'<Twinleaf
fullName of the Benched Pokemon>'}` (Mew ex's own and other attacks keep
`{a:'attack', name}`), so two Benched Pokemon with a same-name attack are both
offered. Options sort by `name` then `from` (the key `name + "\0" + from`, JS string order).
Scripted answers must give `from`; `legal` assertions may omit it (any source) or
give it (an English key or full name). A lock such as "this Pokemon can't use X"
applies to Mew ex for a name among its copied attacks; copy attacks (Foul Play,
Night Joker, Metronome) still don't lock the copied name (Advanced Rulebook C-18).

#### `expect`: the scenario asserts the rules outcome

"0 diverged" only says the Rust engine and the oracle agree. A scenario can
also assert what the rules say with an optional top-level `expect` list. It is
evaluated by the Rust replay (`diff`) only; the oracle ignores the key.

```json
"expect": [
  {"at": "turn_end", "who": "opp", "slot": "active", "card": "Pikachu ex ASC 57",
   "damage": 190, "hp_left": 10, "cite": "ruling 1589"},
  {"at": "turn_end", "who": "me", "bench": 1, "energy": ["Water Energy MEE"], "cite": "rulings 1625, 1651"},
  {"who": "opp", "zone": "discard", "count": 1, "contains": ["Sacred Charm PFL 93"], "cite": "card text"},
  {"winner": null, "cite": "<ruling or rulebook rule>"}
]
```

* **`cite`** (mandatory, a non-empty string): the ruling or rulebook rule that
  makes this the right outcome. An assertion without one is rejected
  (`check_cards.py` exits, `diff` reports EXPECT FAILED), as are unknown keys.
* **`at`**: when it is checked. `"turn_end"`: when the scenario turn's player
  ends the turn, after the attack and its effects are done and Knock Outs are
  resolved (Prizes taken, promotions chosen), before Pokémon Checkup (poison,
  burn, sleep). It is evaluated in the engine's `after_end_turn`, the point
  between the Knock Out check and the start of Checkup. `"next_turn"`
  (default): the first turn decision of the following turn, after Checkup.
  `"next_turn_end"`: the end of the turn after the scenario turn, like `turn_end`, before its Checkup (the other player's
  attack and its Knock Outs; what the opponent did or could not do, e.g. an effect that stops it playing Items). `"game_end"`: the moment the game is decided, winner set (use it for `winner`; a game that never ends
  is not checked). `"tiebreaker"`: the first turn decision of the Tiebreaker game that replaces the scenario's game.
  `"start"`: right after the scenario edits, at the first decision (same as `"decision"` with `n` 0): use it with
  `legal` or `bench_count` for what the edited board allows. `"decision"` + `n`: the n-th turn decision since the edits.
  Every game must satisfy every assertion. A game that ends before the check
  point (a win at turn end is still seen by `turn_end`, but not by
  `next_turn`) is reported as not checked, never as passed.
* **`turn`** (a number >= 0, default 0 with `turn_end`): the check fires at turn `scenario turn + turn` (game
  turns: 2 = the scenario player's next turn). For effects that last into a later turn ("during your next turn, the
  Defending Pokemon takes 100 more damage"): script the turns with `answers` (`{"a": "pass"}` for the other player).
  A game that ends before that turn is reported as not checked.
* **Players**: `who` is `me` (the scenario's first side, the player whose turn
  it is at `turn`) or `opp`. Needed by every assertion except `winner`.
* **Pokémon slot** (pick one selector, then any checks):

  | Selector | Meaning |
  | --- | --- |
  | `"slot": "active"` | The Active Pokémon |
  | `"bench": N` | Bench spot N, left to right from 0, as the engine stores them (a Pokémon switched or promoted keeps its spot; the spot it left is taken by the other one) |
  | `"card": "Name"` | The Pokémon in play whose stack holds that card (any evolution stage). With `slot`/`bench`, `card` is a check instead |

  | Check | Meaning |
  | --- | --- |
  | `damage`: N | Exact damage on the Pokémon |
  | `hp_left`: N | HP (as last computed, with HP changes from Tools and Stadiums) minus damage |
  | `energy`: N or `[names]` | Count, or exactly these Energy cards (any order) |
  | `tool`: name or `null` | The Tool attached, or none |
  | `conditions`: `[...]` | Exactly these Special Conditions (`[]` = none) |
  | `card`: name | Name of the top Pokémon card (with `slot`/`bench`) |
  | `in_play`: bool | `false` = nothing there (or the named card is not in play); a selector that finds no Pokémon fails every other check |

* **Zones**: `{"who", "zone": "hand|deck|discard|prizes|lost_zone", "count": N,
  "contains": [names], "not_contains": [names]}`. `contains` needs distinct
  cards (two names = two cards). `prizes` is the Prize cards still in the Prize pile.
* **Bench and deck**: `{"who", "bench_count": N}` (Pokemon on the Bench), `{"who", "bench_excludes": [names]}` (none of
  these on the Bench), and `"top": [names]` with a `deck` zone (the top cards of the deck, first = top).
* **Other**: `{"who", "prizes_taken": N}` (Prizes taken so far by Knock Outs);
  `{"winner": "me"|"opp"|"draw"|null}` (null = game still going);
  `{"who", "active": "Name"}` (the Active Pokémon's name).
* **Names** (cards, attacks, Abilities) are official or Twinleaf names; both are accepted. Official is preferred.
* **Later turns and legal actions** (rules audit): `turn` is described above. `"at": "decision", "n": N` checks the N-th turn
  decision since the edits (0 = right after them, `"at": "start"`; counted over all later turns), so what the first
  decision of a turn allows can be asserted. `{"who", "legal": KIND, "is": false}` asserts that an action is (not) among
  the legal turn options of the player to move (what the RL action mask may contain; `who` must be that player; `is`
  defaults to true). KIND is `play` (a card in hand, `name`; Energy cards too; `on`: `"active"` or a Bench index narrows
  it to that target), `ability` (`name` of the Ability and/or `card` = the Pokemon that has it), `stadium`, `retreat`
  (`on`: the Bench index) or `attack` (`name`). Scripted `answers` may name a hand card in a play (`{"a": "play", "card": "Switch 30C 127",
  "target": {...}}` or `"name"` instead of `"card"`; `target` optional; or `{"a": "play"|"ability", "card_prefix": "PRE-100#"}`):
  the oracle matches the first such legal option, since card ids depend on the shuffle. Other answers (`ability`, `stadium`,
  `retreat`, `attack`, `pass`) and prompt answers stay in the recorded raw format (e.g. `[{"player": 2, "slot": 2, "index": 0}]`).
* **Prompt order** (`"prompts"`, question timing in attacks): `{"at": "turn_end", "prompts": ["CoinFlip", "PutDamage"], "cite": "..."}`
  asserts that the Rust engine created prompts of these kinds, in this order (a subsequence), since the scenario edits.
  The names are the `PromptKind` variants (`Wait`, `CoinFlip`, `Confirm`, `ChooseCards`, `ChoosePokemon`, `PutDamage`,
  `AttachEnergy`, `ShuffleDeck`, ...); no `who`. Use it to pin when an attack asks its question relative to a coin flip
  or another prompt (rules: Advanced Rulebook A-01 steps 3 to 5, C-07; rulings 1553, 1580, 1770, 1846, 1874).
* **Output**: `diff` prints `EXPECT FAILED <trace>: assertion #i (at) ...`
  with the cite and the actual value, and `expect: N games checked, M failed, K
  not checked`; it exits 1 on failure, like a divergence, with `--quiet` too.
  With `--dump DIR` it writes the Rust state at the check point as
  `<trace>.expect<i>.rust.json`. `check_cards.py` repeats the summary line and
  the first failure, and `run_scenarios.py` counts expect failures in its summary.

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
   `--remote` on Actions, once the branch is pushed).
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
   **Attacks are the exception to "does nothing = unplayable"** (rulings 1790,
   336, 337): an attack can always be used unless its text says "You can't use
   this attack ..." (Illumise, Terapagos ex, Team Rocket's Mewtwo ex ...); when its
   effect can't be carried out (no Bench, no Pokemon in the discard pile, an
   empty deck, a full Bench) it is still used and does nothing, so return
   instead of throwing. The search prefabs
   `SEARCH_YOUR_DECK_FOR_POKEMON_AND_PUT_ONTO_BENCH` / `_INTO_HAND` do that
   themselves during the ATTACK phase (they still throw for Abilities and
   Trainers: an empty deck or a full Bench is public knowledge, ruling 779).
   An Ability, unlike an attack, can't be used for no effect (rulings 12, 244,
   1783): throw CANNOT_USE_POWER when the effect can't happen for a reason
   everyone knows (empty deck for a search or a draw, full opposing Bench for
   "put onto your opponent's Bench" (rulings 46, 70, 1634), an empty hand, no Energy
   to move); a search whose deck may hold no valid card is still usable.
   Choice sizes (rulings 1721, 1778, 1853): "up to N" in an attack takes 0..N;
   in an Ability or a Trainer it takes 1..N (decline = don't use it); "any
   number / any amount" takes 0 anywhere; "N" is exactly N (or as many as
   there are); a deck search for a kind of card may always find fewer or none
   (rulings 519, 839), but a search for "a card" of any kind must take at least
   1 when the deck has any (rulings 892, 1778).
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

8. **Copy sessions** (`copy-attack-delegation.ts`, `copy_attack.rs`) run the
   source card's `reduceEffect` with `this` = the copycat for every effect of
   the copied attack's lifecycle, up to 4 EndTurns later. The copycat's
   `attacks` are the source's clones and a PowerEffect probe for it throws
   BLOCKED_BY_EFFECT (it gets the attack, not the Abilities), but its `powers`
   array is its own (empty for Zoroark, Ethan's Sudowoodo): source code must
   not read `this.powers[0]` outside a branch that checks `IS_ABILITY_BLOCKED`
   first. Crustle BLK's `reason: this.powers[0].name` and Froslass's
   Freezing Shroud count did, and the TypeError ended the game (Y2-1); the
   ports use the literal ability name. A sweep of every pool attack copied by
   Zoroark and Clefable (a scenario per source card, 16 turns of random play)
   is how the remaining sites were ruled out.
   Three more rules of the copy prefabs (Y2-2): a non-cancellable
   ChooseAttackPrompt whose every attack is blocked (Metronome copying
   Accelerating Stab twice running: the copycat can't use it next turn) is not
   opened, the copy does nothing (`noAttackLeftToCopy`, `no_attack_left_to_copy`);
   `COPY_OPPONENT_ACTIVE_ATTACK` (Zoroark's Foul Play) ends the copy silently when
   the chosen attack throws (Follow Me with no Benched Pokémon), as
   `COPY_ATTACK_FROM_POKEMON_LIST` does; and an attack that asks for a resource
   the copycat doesn't have must not open a prompt without a valid answer
   (Inferno X copied by a Pokémon with no [R] Energy).
9. **Move/RemoveDamagePrompt results** are one (from, to) pair per damage
   counter and have no upper bound in Twinleaf (the bot answers with 20-32
   transfers); `Res::DamageTransfers` stores runs `(from, to, count)` (at most
   `MAX_DAMAGE_RUNS`, 32; the bot's answers have at most 17) and
   `damage_transfers` expands them for the card code (Alakazam TWM, Munkidori).

10. **Trainer effects on a Pokémon survive moving and evolving** (rulings 1730,
    1259, 1149, 1150; R7C). Attack effects end when the Pokémon moves to the Bench,
    switches, evolves or devolves, but an effect of a Trainer card or an Ability
    does not. `Marker.removeAllExceptTrainerEffects` /
    `markers.remove_all_except_trainer_effects` is what `clearEffects` /
    `clear_effects` and the two evolution marker wipes now do: markers added with
    `sourceType` `'trainer'` (`SourceType::Trainer`) stay, everything else goes. A
    slot that is vacated is still reset completely (`resetEmptyPokemonSlot`). The
    only pool users: Acerola's Mischief (the protection marker) and Heavy Baton's
    `HEAVY_BATON_ACTIVE_MARKER`; add the source type when a new card keeps a marker
    on a Pokémon for a Trainer effect.
11. **A Supporter's effect used as the effect of an attack** (Mr. Mime's
    Look-Alike Show; rulings 443, 599, 1727, 1728, 1729, 1844, 1853).
    `TrainerEffect.usedAsAttackEffect` (Rust `Effect::Trainer.via_attack`,
    `trainer_via_attack(g, e)`) is set by the attack. Supporter limits don't apply
    (the attack zeroes `supporterTurn` around the effect); the card is not played
    from the hand, so the played-from-hand trackers `rocketSupporter` (Team Rocket's
    Archer, Ariana, Giovanni, Petrel, Proton) and `ancientSupporter` (Explorer's
    Guidance) are not set; the card is in the opponent's hand, so counts of "other
    cards in your hand" must exclude it explicitly (Kofu); an "up to N" prompt over a
    public zone that has min 1 when played may choose zero (Eri, Lana's Aid, N's Plan,
    Gwynn). The Supporter's own text conditions ("You can use this card only if ...",
    costs) still apply (rulings 444, 598, 441): such a Supporter throws and the
    attack's prompt blocks it. Any new Supporter whose prompt minimum is raised for
    an "up to" over a public zone must use `effect.usedAsAttackEffect ? 0 : n`.
12. **Transformation Tome** puts the discard Basic onto the slot first and discards
    the old bottom card after, so the slot is never empty (emptying it discards
    the attachments and resets the damage and conditions). The new card replaces
    the old one at the bottom of the stack and takes over the state kept on the
    card object (`damageTakenLastTurn`, `movedToActiveThisTurn` and the player's
    `movedToActiveThisTurn` / `movedFromActiveToBenchThisTurn` id lists; ruling 1840).
13. **Choices and hidden information** (Rule Book; rulings 1778, 1853, 1721, 386,
    932, 1097, 851, 779, 336, 337). Cards must follow these whatever Twinleaf did:
    * "Up to N" in a Trainer or an Ability: at least 1 when able (`min: 1`, and no
      `allowCancel`, which is the same as choosing 0). Only an attack may choose 0
      (ruling 1721). That holds for public zones (discard pile, cards in play, a
      revealed hand) and for cards you looked at (Hassel, Grimsley's Move). A
      Supporter whose effect is used through an attack (Mr. Mime's Look-Alike
      Show, ruling 1844) may choose 0: the cards test `!effect.usedAsAttackEffect` (section 11)
      at the start of the effect (`played_from_hand = !trainer_via_attack(g, e)` in Rust, kept in the frame).
    * "N" without "up to": exactly N, or as many as you can. "Any number" /
      "any amount": 0 is allowed.
    * A search of the DECK for a card of a given kind may find nothing (the deck
      is hidden): `min: 0`, and the search is always made (prompt opened, deck
      shuffled afterwards) when the deck is not empty, even if it holds no valid
      card (Telepathic Psychic Energy used to skip both). A search for "any
      card" with no kind named must take at least 1 (rulings 1778, 325, 892), and
      a "Look at the top N cards ... put up to X" choice over looked-at cards is
      not hidden.
    * A Trainer, Stadium or Item can't be played or used when it is obvious that
      it would do nothing: an empty deck for a search, a draw or a discard-hand
      cost (rulings 779, 1037, 1098, 1733), nothing to take in the discard pile
      (948), an empty opposing hand (880), a hand already at the target size
      (959), a full Bench for "put onto your Bench" (337). Throw
      `CANNOT_PLAY_THIS_CARD` / `CANNOT_USE_STADIUM` before any state change. A
      deck that only might hold nothing is not an obvious case.
14. **Core rules added in the rulings review (R7F).** (a) *Attack costs*:
    an effect that sets the cost ("can use the attack for [C]": Kyurem,
    Azumarill) or ignores it (Conkeldurr, Decidueye ex) sets
    `CheckAttackCostEffect.setCost` / `ignoreColorless`; the core applies it
    after every handler, so no increase (Rillaboom, Nighttime Mine, Antique
    Root Fossil, ...) or decrease (Counter Gain, ...) touches it, in either
    handler order (rulings 147, 252, 1552, 1581, 1842; the Rust fields are
    `set_cost` / `ignore_colorless`). (b) *AfterAttackEffect* reaches Pokémon,
    then Energy, then Trainers, so effects triggered on the Defending Pokémon
    (Handheld Fan) resolve after the attack's own effects and after Boomerang
    Energy re-attaches (rulings 1625, 1650); such a trigger arms a marker on
    the attacker's slot in AfterDamageEffect and resolves in AfterAttackEffect.
    A Stadium an attack discards goes in AfterAttackEffect too (after the
    damage, before the Knock Out check: rulings 1559, 1589). (c) *Knock Outs*:
    the Check State step announces every KnockOutEffect before any Pokémon
    leaves play (`deferRemoval` / `completeKnockOut`, Rust `defer_removal` /
    `complete_knock_out`), so an Ability that reacts to a Knock Out still works
    for a Pokémon Knocked Out at the same time (Togekiss, ruling 1623).
    (d) *Winning*: taking the last Prize card does not end the game by itself;
    `checkWinner` counts both players' win conditions (no Prize cards left, no
    Pokémon in play) and the player with more wins, equal numbers go to Sudden
    Death (rulings 234, 820, 1403); when both players took their last Prize
    the new Active Pokémon are promoted first (ruling 1584). The player whose
    turn is next takes Prizes and promotes first (rulings 754, 757), and Prize
    reductions never go below 0 (ruling 1745).

Report per fix: list number, card (international key), Twinleaf commit,
Rust change, scenario entry (format above), the official text it now
follows, and the diverged traces with their causes.


### Attack text runs before the damage: what must wait (2026-10-07)

Both engines run a card's attack text when its `AttackEffect` is reduced, which is BEFORE the damage. The flow chart puts
the attack's effects (step 5) AFTER the damage calculation. Chart order is kept only by targeted deferrals (Energy
removal via the after-damage window, effects coded in `AFTER_ATTACK`, the step-6 trigger queue). So when porting or
reviewing an attack, ask: **does this text change anything the damage calculation reads?** That includes:
- the Defending Pokemon's Tools, Abilities or effects;
- its damage counters, or whether it has full HP;
- attached Energy on either Pokemon;
- which Pokemon is Active.

If it does, and the text doesn't say "before doing damage", it must resolve after the damage (after-damage window or
`AFTER_ATTACK`), in both engines. The same goes for text that depends on the damage just done (heal the damage dealt,
"if the Defending Pokemon is Knocked Out"). This is a stopgap: a Rust-only refactor to an explicit step pipeline is
planned after the oracle freeze.

### Attack flow chart steps 6-8 (F1)

Official order (attack flow chart; Advanced Player's Rulebook A-01, E-03, E-04): step 5 the attack's own effects, step 6
effects that activate when the Defending Pokemon is damaged (Spiky Energy, Punk Helmet, Lucky Helmet, Handheld Fan,
Heatran, the delayed traps), step 7 the Knock Out check (Knock Out triggers such as Maractus come at its step 2, just
before the discard). Card code follows it like this, in both engines:

* A step 6 card reacts to AfterDamageEffect only to *record* the trigger: `ATTACK_TRIGGER(store, state, effect, card)`
  in Twinleaf (`prefabs/after-damage.ts`), `g.attack_trigger(b, damage, card, retaliate, removes_attacker_energy)` in
  Rust. The trigger resolves after AfterAttackEffect and the prompts it opened, as an `AttackTriggerEffect`
  (`Effect::AttackTrigger`, kind 246) that only `card` reacts to. At resolution the card re-checks everything that can
  have changed since the damage: still attached to the damaged Pokemon, not blocked (Ability, Tool, Special Energy),
  and `sourceInPlay` for the Attacking Pokemon (it can be on the Bench or gone).
* Triggers resolve one at a time in the order recorded (cards react in Pokemon, Tool/Trainer, Energy order, then the
  core's delayed traps); a trigger that opens a prompt is answered before the next. When 2+ are pending and the order
  matters (`ordersMatter` / `trigger_order_matters`: Handheld Fan next to a delayed trap while the attacker holds a Mist
  Energy) the defending player chooses which goes first with a Select prompt.
* Knock Out triggers use `prefabs/last-attack.ts` (`game.last_attack`): the Pokemon that used the attack and the opponent's
  Pokemon it damaged in the Active Spot, kept until the check. Use `ATTACKER_OF_KNOCK_OUT` / `ATTACK_THAT_DAMAGED_KNOCKED_OUT`
  (`attacker_of_knock_out`) instead of "the opponent's Active Pokemon", which is another Pokemon after a switch.
