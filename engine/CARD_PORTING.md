# Adding or fixing a card

Every card with logic is a declarative spec: a `static CardSpec` in
`engine/src/cards/impls/<name>.rs`, run by one interpreter in
`engine/src/spec/`. The Rust engine is the reference implementation (the
Twinleaf TypeScript oracle is frozen and no longer part of the project). The
rules come from, in this order of authority: judge rulings
(`data/rulings/all.json`), the official card text (`data/official_text.json`),
the rulebooks, and `engine/RULES.md` (decisions already made, with citations).

## Where things are

| What | Where |
| --- | --- |
| Card specs | `engine/src/cards/impls/*.rs`, one file per behavior class; `build.rs` registers them |
| Interpreter | `engine/src/spec/run.rs` (programs, frames, the step-D choice pass) |
| Ops (the verbs of a spec) | `engine/src/spec/ops/{cards,board,flow,state}.rs` |
| Passive modifiers, triggers | `engine/src/spec/passive.rs`, `engine/src/spec/trigger.rs` |
| Conditions, numbers, predicates, selectors | `engine/src/spec/value.rs` (`Cond`, `Num`, `Pred`, `SlotPred`, `SlotSel`) |
| Spec types (`CardSpec`, `Step`, `Op`, `Once`) | `engine/src/spec/mod.rs` |
| Core rules | `engine/src/engine/*.rs`, store in `engine/src/game.rs`, effects in `effects.rs`, prompts in `prompts.rs`, state in `state.rs` |
| Card data | `data/cards.json`: every card's printed fields, keyed by its international key (`"<Name> <SET> <NUM>"`); the file's order is the card ids, so append new cards at the end. `data/pool.json` lists the pool (a row per card, with its key); `data/support_cards.json` the pre-evolutions outside the pool |
| Card database (generated, never edit) | `engine/src/gen/*.rs`, from `data/cards.json` and `data/evolutions.json` by `tools/gen_carddb.py` |
| Scenarios | `scenarios/*.json`, run by `engine/src/bin/scen.rs` |
| Rules decisions | `engine/RULES.md` |
| Vocabulary reference | the doc comments in `engine/src/spec/*.rs`; the files of real cards are the best examples |

## Writing a card as a spec

A card file has the printed text as a header comment, one `SPEC` and one
`IMPL`:

```rust
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "BuddyBuddyPoffin",   // the behavior class: CardDef::behavior in gen/cards.rs
    play: Some(PlaySpec { kind: PlayKind::Item, needs: &[], steps: &[ /* ... */ ] }),
    ..CardSpec::NONE
};
pub static IMPL: CardImpl = SPEC.card_impl();
```

All logic goes in `engine/src/spec`, never in a card file. If the vocabulary
lacks something, add it to the interpreter (an op, a `Cond`/`Num`/`Pred`
variant, a modifier or an event), with a scenario, then use it. Write effects
in printed order.

### The parts of a `CardSpec`

| Field | For | Notes |
| --- | --- | --- |
| `attacks: &[AttackSpec { index, steps }]` | attack text | `index` is the attack's position on the card. Steps are `Step::before_damage(op)` or `Step::after_damage(op)` |
| `powers: &[PowerSpec { index, once, needs, steps }]` | activated Abilities | `once`: `Once::No`, `PerTurn("MARKER")` (per copy) or `PerTurnShared("MARKER")` (all copies, "1 X per turn"); the marker is set by the card and cleared at end of turn |
| `play: Option<PlaySpec { kind, needs, steps }>` | Item, Supporter, Tool, Stadium effect when played | `PlayKind::{Item, Supporter, Tool, Stadium}` |
| `use_stadium` | a Stadium's "once during each player's turn" use | same shape as `play` |
| `passives: &[Passive { origin, modifier }]` | continuous effects | `Modifier` variants: `DamageDealt`, `DamageTaken`, `PreventDamage`, `Prevent`, `BlockUse`, `AbilityLock`, `AttackCost`, `RetreatCost`, `HpBonus`/`HpMod`, `SurviveOnTen`, `ProvidesEnergy`, `GrantAttacks`, `BlockAttack`, `ConditionImmunity`, ... (`passive.rs`) |
| `triggers: &[Trigger { origin, event, steps }]` | "when ..." effects | `Event`: `OnEnterPlay`, `OnMoved`, `OnAttach`, `OnKnockOut`, `OnDamagedByAttack`, `OnCheckup`, `OnEndTurn`, `OnDiscarded`, `OnAfterAttackTriggers` (`trigger.rs`) |

`origin` (`RuleSource::{Ability, CardRule, ...}`) says what the effect is: an
Ability is turned off by Ability locks, a card rule is not.

Ops are grouped by family: moving cards and Energy (`Move`, `Pick`, `Draw`,
`Search`, `Shuffle`, `Attach`, `MoveEnergy`, `DiscardEnergy`, Prize ops),
board (`Damage`, `EachSlot`, `PlaceCounters`, `Heal`, `Switch`, `Conditions`,
`KnockOut`, `Evolve`, `RemoveFromPlay`, ...), flow (`May`, `If`, `Coin`,
`Choose`, `ForEach`, `Fail`, `PickAttack`, `CopyAttack`, `EndTurn`,
`EndGame`) and state (`AttackFlag`, `SetMarker`, `ClearMarker`, `Arm`,
`AbilityUsed`, `SetFlag`). Read the enum in `spec/mod.rs` and the doc comment
of the record each variant takes; use `..Spec::DEFAULT` for the fields you
don't need.

### Attacks: before and after the damage

Use `Step::before_damage` only for text that changes the attack itself before
the damage is calculated (damage modifiers, "this attack does 30 more damage
for each ...", Shred's flag). Every other effect is `Step::after_damage`,
written in printed order. The interpreter does the rest:

* Every choice an attack requires is made at step D, before the damage, and
  carried out after it. The interpreter's choice pass asks the op's `choice`
  half before the damage and keeps the answer (`Game::spec_choices`); the
  after-damage run uses it. Spec authors do nothing special.
* A search, or a choice whose options only exist after the damage, is asked
  after the damage (it has no step-D half). Choices nested under a coin, an
  `If` or a loop are asked when reached.
* A `May` ("you may ...") asks at step D too, when the effect could happen.

Example (Cynthia's Garchomp ex, attack 1: `Corkscrew Dive - 100; you may draw
until you have 6 cards`, attack 2 discards all its Energy):

```rust
AttackSpec { index: 0, steps: &[Step::after_damage(Op::May(MaySpec {
    asker: Who::Me,
    when: Cond::All(&[
        Cond::Cmp(Num::ZoneSize(ZoneRef(Who::Me, Zone::Hand)), CmpOp::Lt, Num::Lit(6)),
        Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any),
    ]),
    msg: "WANT_TO_DRAW_UNTIL_6",
    yes: &[Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::UntilHandSize(Num::Lit(6)) }))],
    no: &[],
}))] },
AttackSpec { index: 1, steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec {
    target: SlotTarget::Slot(SlotExpr::Active(Who::Me)),
    selection: EnergySelection::AllProvided, ..DiscardEnergySpec::DEFAULT }))] },
```

### Trainers and Abilities: legality

Whether a card can be played or an Ability used is declared, not discovered
by running the card:

1. **`needs`**: a list of `Cond`s that must all hold (a Stadium that must
   differ from the one in play, "you must have a Benched Pokémon", a turn
   restriction).
2. **Implied checks of the ops.** The interpreter asks each top-level op
   (`implied_ok`) whether it can do anything. A Trainer or Ability whose only
   effect is drawing needs a drawable card; a `Search`/`Pick` needs something to choose from (a
   deck search needs a nonempty deck, not a matching card: a search may find
   nothing); `Attach` and `PlayFromZone` need a target;
   a required `Switch`, `Heal` or `PickSlot` needs a Pokémon to choose. Rulings 925 and 2362: a card
   that can have no effect can't be played.
3. **A leading `Fail { unless, error }` step** for a use restriction an op
   can't imply, and for attacks that can't be used ("can't attack unless ...").

Attacks stay usable when their effect can't happen (draw as much as
possible); that is a rule, not an omission. Declared checks are read with the
game's own type and Energy checks. Use `needs` rather than a trial run of the
card.

Example (Buddy-Buddy Poffin; the Search's own check makes it unplayable with
an empty deck):

```rust
play: Some(PlaySpec { kind: PlayKind::Item, needs: &[], steps: &[
    Step::new(Op::Search(SearchSpec {
        pick: PickSpec { chooser: Who::Me, from: ZoneRef(Who::Me, Zone::Deck),
            predicate: Pred::All(&[Pred::Basic, Pred::HpAtMost(70)]),
            bounds: Bounds { min: Num::Lit(0), max: Num::Lit(2) }, ..PickSpec::DEFAULT },
        destination: SearchDestination::Bench, msg: "CHOOSE_CARD_TO_PUT_ONTO_BENCH",
        cancel: false, shuffle_first: false })),
    Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
] }),
```

Other worked examples: `meowthex.rs` (on-play trigger with `May`, a marker and
its end-of-turn clearing), `team_rockets_arbok.rs` (a `BlockUse` passive and an
attack on every opposing Pokémon with `EachSlot`), `jolteonex.rs` (before-damage
discard that raises the damage with `Num::Last`, `Arm` for a lasting effect, a
Tera passive), `cynthias_garchompex.rs`.

### `Op::Custom`: almost never

`Op::Custom` (and `Event::Custom`) run hand-written Rust inside a spec. Today
only two cards use them: Mr. Mime (Look-Alike Show: use a Supporter from the
opponent's revealed hand, with a retry session when the chosen Supporter can't
be used) and Backtrack Badge (a coin re-flip session on the attacker's flips).
Their rules need an interactive session that no op expresses. Do not add a
new Custom card: add the missing op, predicate or event to the interpreter
instead. A Custom op that asks questions suspends through a `CardFrame` and
continues in its `resume` function; copy `mr_mime.rs` if you ever must.
A few cards also keep a small coin-result hook (`CardImpl::coin`); prefer `Op::Coin`.

### Reprints and pins

Reprints are the same card: `CardDef::behavior` resolves every printing to one
behavior class, so one unpinned spec covers them all. Pin a spec to one
printing only when two different cards share a class name: `class:
"Class@SET"`, `"Class@Full Name"` or `"Class@A|B"` (see `class_matches` in
`engine/src/cards/mod.rs`; the set and name are the card database's
`set` / `full_name`, e.g. `Class@PFL` or `Class@Hoothoot SCR 114`). Pins
exist today, for example `Minccino@SSH`, `Hoothoot@TEF`.

### Names

The engine speaks official English names: `CardDef.full_name` is the
international key (`"<Name> <SET> <NUM>"`, as in `data/pool.json`),
`CardDef.name` / `AttackDef.name` / `PowerDef.name` are the official names.
Rules wording ("a card named X") compares `name`; write official names in
literals. Every input names cards by the key (decks, scenarios, scripted
answers, tools); `"Name SET"` also works when only one card has it. Attacks
and Abilities are named by their printed names. Cards outside the pool (older
printings, support pre-evolutions) have keys built the same way from their
set and number.

## Rules that shape every spec

Decisions beyond the card text live in `engine/RULES.md` (Ability locks and
their precedence, on-play Abilities, ...). Add new decisions there with their
ruling. A ruling's principle applies to every card with the same pattern, not
just the card it names; when you fix one card, search the pool for the others.

### Attack flow chart

An attack runs in this order (attack flow chart; Advanced Player's Rulebook
A-01, E-03, E-04):

1. Declare the attack and make every choice the attack requires (step D).
2. Attack text that changes the attack, then damage calculation: Weakness,
   Resistance, effects on both Pokémon, then the damage is placed.
3. The attack's effects, in printed order, carried out with the choices made
   at step D.
4. Effects that trigger because the Defending Pokémon was damaged: Spiky
   Energy, Punk Helmet, Lucky Helmet, Handheld Fan, Heatran, delayed traps.
5. Knock Out check: Knock Out triggers (Maractus) come just before the
   discard, Prizes are taken, new Active Pokémon are chosen.
6. Pokémon Checkup after the turn ends.

Consequences in the code:

* Text that reads or changes anything the damage calculation reads must wait
  until after the damage unless the text says "before doing damage": the
  Defending Pokémon's Tools, Abilities or effects, its damage counters (and
  whether it has full HP), attached Energy on either Pokémon, which Pokémon
  is Active. So is text that depends on the damage just done ("heal the
  damage dealt", "if the Defending Pokémon is Knocked Out"). Write it as
  `Step::after_damage`.
* Damage triggers (`attack_trigger` in `game.rs`) are recorded when the damage
  is done and resolve after the attack's own effects and the prompts they
  opened, one at a time in the order recorded. Each re-checks everything that
  may have changed: still attached, not blocked, the attacker still in play.
  When two are pending and the order matters (`trigger_order_matters`: Handheld
  Fan beside a delayed trap while the attacker holds a Mist Energy), the
  defending player chooses the order.
* Knock Out triggers use `game.last_attack` (the Pokémon that attacked and the
  one it damaged in the Active Spot, kept until the check). Use that, not
  "the opponent's Active Pokémon", which can be another Pokémon after a
  switch.

### Energy removed as an effect of an attack

Energy that leaves a Pokémon as an effect of an attack (discarded, shuffled
into the deck, put into the hand, moved to another Pokémon) leaves after the
damage: the player chooses the Energy at step D, the damage is done with the
Energy still attached (Double Turbo Energy, Voltaic Lightning Energy, Spiky
Energy, Ancient Bulwark), then the Energy is removed (rulings 1553, 1580, 1846,
1874). The `DiscardEnergy`, `MoveEnergy` and `Move` ops in an after-damage step
get this from the after-damage window of the attack (`Game::after_dmg`,
`fx_flag::AFTER_DMG_OPEN`). Exceptions, which are not deferred: a cost in the
attack's cost line, a discard from the hand, and discards of non-Energy cards.
Retaliation of an Energy on the Defending Pokémon (Spiky Energy) happens only
if the card is still attached when the trigger resolves, so Duraludon PFL's
Hyper Beam discarding Spiky Energy stops it (no ruling names this case; the
chart order decides).

### Shred ("isn't affected by any effects on your opponent's Active Pokémon")

An attack with that text (Shred, Demolish, Twin Shotels, Azure Wave, ...) sets
the attack flag with `Op::AttackFlag(AttackFlagKind::IgnoreDefenderEffects)`
as a `before_damage` step (`Effect::Attack.ignore_defender_effects`) and lets
the damage go through the normal damage path; never write damage straight to
a Pokémon. Rulings 1439, 1345, 1629, 1875, 1490: every effect on the damaged
Pokémon that changes this damage is skipped (prevention, reduction, extra
damage taken, coin-flip prevention, Tera/Bench protection, Abilities, Tools,
Stadiums). Rulings 1716, 1816, 531, 532, 812, 941: effects on the attacker
(Maximum Belt, Binding Mochi, "attacks used by the Defending Pokémon do N
less"), Weakness and Resistance (and effects that change them) still apply.
Rulings 936, 1770: "survive on 10 HP" effects still apply after the full
damage. A passive that changes or prevents the damage a Pokémon takes must
not apply to such damage (`ignores_defender_effects` in `prefabs.rs`); an
attacker-side bonus must. Generic "prevent all damage and effects" passives
(Milotic ex, Acerola's Mischief) skip the damage steps only.

## Verify

In the order a person uses them. The tools default to half the cores at low
priority on a Mac (`--threads N`, `--full` to change that); read each tool's
docstring for the rest.

1. **Build and unit tests**: `cd engine && cargo build --release --bins`
   (use `--profile iter` for faster rebuilds while editing), then
   `cargo test --release`.
2. **The card itself**: `python3 tools/check_cards.py "<key>" [--games N]`
   (the key is `"<Name> <SET> <NUM>"`, see `data/pool.json`; a Stage 1/2 card
   needs its pre-evolution among the targets). It builds decks containing the
   card, plays them with `engine/target/release/fuzz` (engine errors, stuck
   prompts, turns with no options, invariant violations and panics fail), prints
   how often the card acted (`scout`: a card that never acted was not tested),
   and runs every scenario that mentions it. Use at most about 9 targets per
   run, or the decks become invalid.
3. **Scenarios for the branches random games rarely reach**: write one per
   branch (empty deck, empty hand, a full Bench, an opponent at 2 Prizes, ...)
   and run it: `check_cards.py "<key>" --scenario scenarios/X.json --games 12`,
   or `engine/target/release/scen scenarios/X.json --games 12`. Give each rule
   outcome an `expect` with a citation (below).
4. **The whole scenario suite**: `engine/target/release/scen scenarios --games 8 --quiet`.
5. **Fresh-seed self-play**: `python3 tools/fuzz.py --games N --seed S` (meta
   decks plus random pool decks). A failing game's trace is kept; replay it with
   `engine/target/release/diff <trace>`.
6. **Behavior-preserving changes** (refactors, a new op that replaces a
   pattern): the golden corpus (`corpus/golden/<commit>/traces`, recorded with
   `fuzz --keep`, not in git) must replay with 0 divergences:
   `engine/target/release/diff corpus/golden/<commit>/traces --quiet`. After an
   intended rules change, re-record the same seeds and review which games
   changed; each change must be explained by the ruling you applied.

CI (`.github/workflows/rust.yml`) runs the tests, every scenario and a fuzz
run.

Card status in a report: **verified** (check passes, the card acted, every
reachable branch has a scenario that ran it in at least 3 games), **partial**
(say which branch was not reached), **blocked** (say what is missing).

## Scenarios

A scenario starts every game from a crafted position: the game runs normally
from seed and decks, and at the first turn decision on or after `turn` the
board edits are applied, then play continues normally.

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
  heads; legality checks are unaffected), `answers` (the next decisions, in
  the trace's answer format, e.g. `{"a": "attack", "name": "Piercing Gaze"}`,
  `{"a": "pass"}`, `{"a": "retreat", "bench": 0}`; prompt answers as recorded
  in traces), `decks`, `sudden_death: true` (the game is a Tiebreaker game).
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
* In `answers`, `{"a": "play" | "ability", "card_prefix": "PRE-100#"}` uses
  any copy of that card (instance ids depend on the shuffle); copy the
  prefix format from a recorded trace.
* `decks` is optional (default: the usual auto decks). Every deck should hold
  the scenario's cards, since either player may be `me`, and must be legal: 60
  cards, at most 4 copies, one ACE SPEC, one Radiant, a Basic Pokémon (a
  Special Energy such as Legacy Energy is an ACE SPEC; use Mist Energy for a
  plain one). An illegal deck is reported as `INVALID DECK`.
* Effects that last ("during your opponent's next turn...", markers) can't be
  written as edits: set the board, then use `answers` to play the real attack
  or Trainer that creates them.
* `answers` can't name a card id, so a branch behind "play this Supporter"
  relies on the policy playing it; keep the hand small so it is the only play.
* "Knocked Out during your opponent's last turn" scenarios: copy
  `scenarios/hassel-ko-previous-turn.json`.
* Example covering most edits: `scenarios/smoke-full.json`.
* One file per card and branch: `scenarios/<card-slug>-<branch>.json`
  (e.g. `luxray-ex-empty-hand.json`). A scenario must reach the branch through
  normal play after the edits (the policy still picks moves): set the board so
  the branch is likely, and run enough games (12-16) that it runs in at least 3.

### Attacks copied by Memory Helix

Mew ex's Memory Helix is passive: while Mew ex is Active and the Ability
isn't blocked, the attacks of its Benched Pokémon are added to its attack
options. The turn option of a copied attack always carries its source
(`{"a": "attack", "name": ..., "from": <the Benched Pokémon>}`), so two
Benched Pokémon with a same-name attack are both offered. Scripted answers
must give `from` (copy the format from `scenarios/fix-mew-ex-memory-helix-attack-throws.json`);
`legal` assertions may omit it (any source). A lock such as "this Pokémon can't
use X" applies to Mew ex for a name among its copied attacks; copy attacks
(Foul Play, Night Joker, Metronome) don't lock the copied name (Advanced
Rulebook C-18).

### `expect`: the scenario asserts the rules outcome

A passing run only says the engine didn't break. A scenario states what the
rules say with an optional top-level `expect` list, evaluated by the replay.

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
  makes this the right outcome. An assertion without one is rejected, as are
  unknown keys.
* **`at`**: when it is checked.
  * `"turn_end"`: when the scenario turn's player ends the turn, after the
    attack and its effects are done and Knock Outs are resolved (Prizes
    taken, promotions chosen), before Pokémon Checkup.
  * `"next_turn"` (default): the first turn decision of the following turn,
    after Checkup.
  * `"next_turn_end"`: the end of the turn after the scenario turn, like
    `turn_end`, before its Checkup (the other player's attack and its Knock
    Outs; what the opponent did or could not do, e.g. an effect that stops it
    playing Items).
  * `"game_end"`: the moment the game is decided, winner set (use it for
    `winner`; a game that never ends is not checked).
  * `"tiebreaker"`: the first turn decision of the Tiebreaker game that
    replaces the scenario's game.
  * `"start"`: right after the scenario edits, at the first decision (same as
    `"decision"` with `n` 0): use it with `legal` or `bench_count` for what
    the edited board allows.
  * `"decision"` + `n`: the n-th turn decision since the edits.

  Every game must satisfy every assertion. A game that ends before the check
  point (a win at turn end is still seen by `turn_end`, but not by
  `next_turn`) is reported as not checked, never as passed.
* **`turn`** (a number >= 0, default 0 with `turn_end`): the check fires at
  turn `scenario turn + turn` (2 = the scenario player's next turn). For
  effects that last into a later turn ("during your next turn, the Defending
  Pokémon takes 100 more damage"): script the turns with `answers`
  (`{"a": "pass"}` for the other player). A game that ends before that turn is
  not checked.
* **Players**: `who` is `me` (the player whose turn it is at `turn`) or `opp`.
  Needed by every assertion except `winner`.
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
  cards (two names = two cards). `prizes` is the Prize cards still in the
  Prize pile.
* **Bench and deck**: `{"who", "bench_count": N}`, `{"who", "bench_excludes":
  [names]}` (none of these on the Bench), and `"top": [names]` with a `deck`
  zone (the top cards of the deck, first = top).
* **Other**: `{"who", "prizes_taken": N}` (Prizes taken so far by Knock
  Outs); `{"winner": "me"|"opp"|"draw"|null}` (null = game still going);
  `{"who", "active": "Name"}` (the Active Pokémon's name).
* **Names** (cards, attacks, Abilities): official English names; cards by
  their key (`"<Name> <SET> <NUM>"`), or `"Name SET"` when only one card has
  it. See "Names" above.
* **Legal actions**: `{"who", "legal": KIND, "is": false}` asserts that an
  action is (not) among the legal turn options of the player to move (`who`
  must be that player; `is` defaults to true). KIND is `play` (a card in hand,
  `name`; Energy cards too; `on`: `"active"` or a Bench index narrows it to
  that target), `ability` (`name` of the Ability and/or `card` = the Pokémon
  that has it), `stadium`, `retreat` (`on`: the Bench index) or `attack`
  (`name`). Scripted `answers` may name a hand card in a play (`{"a": "play",
  "card": "Switch 30C 127", "target": {...}}`, `"name"` instead of `"card"`,
  or `card_prefix`; `target` optional): the first matching legal option is
  used, since card ids depend on the shuffle. Other answers (`ability`,
  `stadium`, `retreat`, `attack`, `pass`) and prompt answers stay in the
  recorded raw format (e.g. `[{"player": 2, "slot": 2, "index": 0}]`).
* **Prompt order** (`"prompts"`, question timing in attacks): `{"at":
  "turn_end", "prompts": ["CoinFlip", "PutDamage"], "cite": "..."}` asserts
  that the engine created prompts of these kinds, in this order (a
  subsequence), since the scenario edits. The names are the `PromptKind`
  variants (`Wait`, `CoinFlip`, `Confirm`, `ChooseCards`, `ChoosePokemon`,
  `PutDamage`, `AttachEnergy`, `ShuffleDeck`, ...); no `who`. Use it to pin when
  an attack asks its question relative to a coin flip or another prompt
  (Advanced Rulebook A-01 steps 3 to 5, C-07; rulings 1553, 1580, 1770, 1846,
  1874).
* **Output**: `scen` and `diff` print `EXPECT FAILED <trace>: assertion #i
  (at) ...` with the cite and the actual value, and a summary `expect: N games
  checked, M failed, K not checked`; the exit status is 1 on a failure.
  `diff --dump DIR` writes the state at the check point as
  `<trace>.expect<i>.rust.json`. `check_cards.py` repeats the summary line and
  the first failure.

The implementation is `engine/src/scenario.rs` (edits) and
`engine/src/expect.rs` (assertions). If a branch needs an edit that doesn't
exist yet (say, setting a Prize card), add it there with a test scenario.

## Rules for contributors and agents

* Rulings decide. When the card text and a ruling disagree, follow the
  ruling; when neither settles it, ask, then record the decision in
  `engine/RULES.md` with its citation. Generalize: a ruling about one card
  applies to every card with the same pattern, so fix them all and add a
  scenario with an `expect` for each distinct case.
* Correct beats faithful to old behavior. When a replay diverges after a
  change, check the rulings before deciding which side is wrong; never revert
  to "what it did before" by default.
* Never edit or rely on Twinleaf (the frozen TypeScript oracle).
* Don't edit `data/verified.json` by hand; report card statuses and the
  merger records them.
* Don't change a card owned by another branch unless the fix needs it, and
  say so.
* One commit per logical change, with the configured git identity and no
  Co-Authored-By or other trailers. Commit only your own files and the core
  changes they need. Don't push unless asked.
* The repository is public: no machine names, accounts, cloud details or costs
  in files, comments or commit messages.
* When several agents build at once, run cargo through the lock:
  `sh porting/heavy.sh cargo build --release --bins` (a local, untracked helper; run it from the main checkout)
  (never `cargo fmt`, never `pkill` by pattern).
