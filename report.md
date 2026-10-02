2026-10-02T20:32:12Z

# Batch c3 report (branch port-c3)

Final commits: 61640fe (ports, core changes, scenarios), 66cb08b (handbook note); report.md is committed after them.
Corpus dirs (gitignored, in the worktree): corpus/cards/c3-a, c3-b, c3-c, c3-d, c3-e, c3-f (parity); c3-a-cov .. c3-d-cov (random coverage, 48 games each, --remote 4); c3-scen-<scenario> (one per scenario file).
Final checks: `diff corpus/t1 + main corpus/cards/*/ + worktree corpus/cards/*/ --quiet`: 33286 traces, 0 diverged (2458 unsupported are other batches' cards, 2 approved). `cargo test --release`: ok. check_pins: ok.

## Card status
Trace counts are from the 16-game parity runs / 48-game coverage runs (all 0 diverged) plus the scenario runs below.
- Flutter Mane PRE 43 (Flutter Mane PRE): verified (random coverage: 0 segments below 3).
- Fezandipiti PRE 45 (Fezandipiti PRE): partial. Ability-blocked, [D] coin heads/tails, Energy Feather verified. Not reached: L43 `pokemonCard !== this || phase !== ATTACK` return and L59 `effect.damage <= 0` return (see exemption candidates).
- Iron Boulder PRE 46: verified.
- Duraludon PRE 69: verified (pin `Duraludon@SCR|PRE`; the SSP and PFL Duraludon are different classes with the same name).
- Carmine PRE 103: verified except canPlay `supporterTurn>0` / `deck empty` UI-only returns (exemption candidates).
- Explorer's Guidance PRE 107: verified except canPlay returns (exemption candidates). Adds `Player.ancient_supporter`.
- Janine's Secret Art PRE 112 (Janine's Secret Art PRE, behavior JaninesSecretTechnique): verified except canPlay return (exemption candidate).
- Rescue Board PRE 126: verified except the tool-blocked `return state` (exemption candidate).
- Scoop Up Cyclone PRE 128: verified except the `: null` ternary fallback (exemption candidate).
- Sparkling Crystal PRE 129: verified except the ToolEffect-stub `catch` (exemption candidate).
- Ceruledge ex PRE 147 (Ceruledge exSAR PRE): verified.
- Iron Crown ex PRE 158 (Iron Crown exSAR PRE): verified except `if (targets == null) return state` (exemption candidate).
- Drayton PRE 172 (DraytonSAR PRE): verified except canPlay return (exemption candidate).
- Walking Wake ex PRE 178 (Walking Wake exUR PRE): verified (needed per-card shredAttack canonical field).
- Bronzor SSP 126: verified.
- Scramble Switch SSP 186: verified (energy move, no energy, no bench, cancel).
- Cook TWM 147: verified.
- Cofagrigus WHT 40: partial. Reached: damaged-bench prompt chain, MoveDamageCounters prevented (Patrat), Perplex, Mist Energy interplay. Not reached (exemption candidates): empty-selection returns, `damageToMove <= 0`, `source.damage < 0` clamp.
- N's Darmanitan ASC 33: verified (but see stuck-prompt bug).
- Pikachu ex ASC 57: verified (pin `Pikachuex@SSP|ASC`).
- Slowpoke MEP 86: verified.
- Support cards (no coverage report exists for non-pool cards; verified by 16-game parity runs c3-e and the scenarios): Shelgon JTG (`Shelgon@JTG`), Torracat TEF (`Torracat@TEF`), Antique Cover Fossil SCR, Galarian Zigzagoon FST 159 (`GalarianZigzagoon@Galarian Zigzagoon FST 159`): all 0 diverged. `data/support_cards.json` already listed all four (vanilla false); no change.

## Scenarios (all 0 diverged; each also run with --coverage)
scenarios/carmine-empty-deck.json - Carmine PRE 103
  targets:  carmine.ts:40 `throw CANNOT_PLAY_THIS_CARD` (empty deck)
  setup:    reset, hand Carmine, rest of deck in discard/prizes so the deck is empty
  result:   12 traces; not listed below 3 games (line 40 gone from the report)
  corpus:   corpus/cards/c3-scen-carmine-empty-deck
scenarios/carmine-supporter-played.json, explorers-guidance-supporter-played.json, janines-secret-art-supporter-played.json, drayton-supporter-played.json
  targets:  canPlay `supporterTurn > 0` return (carmine.ts:26, explorers-guidance.ts:24, janines-secret-technique.ts:24, drayton.ts:83)
  setup:    `supporter_played: true` with the Supporter in hand
  result:   12 traces each, 0 diverged; target lines still 0 games (FAILED to cover: the oracle never calls canPlay of a Supporter once one was played). Kept as evidence for the exemptions.
  corpus:   corpus/cards/c3-scen-<file>
scenarios/scramble-switch-energy-move.json - Scramble Switch SSP 186
  targets:  scramble-switch.ts L25 cancel return, L30-35 energy prompt + move + switch
  setup:    reset; Active with 2 Metal Energy, 2 Benched, Scramble Switch in hand
  result:   150 traces (seed 200000), 0 below 3 games (cancel branch needed 150 games; 48 gave 2)
  corpus:   corpus/cards/c3-scen-scramble-switch-energy-move
scenarios/scramble-switch-no-energy.json: Active without Energy -> direct switch (L35 `player.switchPokemon`), 12 traces, covered. corpus c3-scen-scramble-switch-no-energy
scenarios/scramble-switch-no-bench.json: no Bench -> L14 `throw CANNOT_PLAY_THIS_CARD`, 12 traces, covered. corpus c3-scen-scramble-switch-no-bench
scenarios/scoop-up-cyclone-board.json - Scoop Up Cyclone PRE 128: hand Scoop Up Cyclone, Active+2 Bench with Energy/damage; 12 traces; effect and prompt covered (only `: null` left). corpus c3-scen-scoop-up-cyclone-board
scenarios/sparkling-crystal-terapagos.json (+ -one-energy) - Sparkling Crystal PRE 129: Terapagos ex ASC Active with the tool, Grass + Prism Energy (rainbow); covers the whole Tera branch (colorless, typed, rainbow slots, cost-1 test); only the `catch` remains. 12 traces. corpus c3-scen-sparkling-crystal-terapagos(-one-energy)
scenarios/ceruledge-ex-amethyst-rage.json - Ceruledge ex PRE 147 / Pikachu ex ASC 57: Ceruledge with R/P/M Energy uses Amethyst Rage (answers) on a full-HP Pikachu ex; covers ceruledge-ex.ts L47-55 and pikachu-ex.ts L44 (survive at 10). 12 traces. corpus c3-scen-ceruledge-ex-amethyst-rage
scenarios/pikachu-ex-topaz-bolt.json - Pikachu ex Topaz Bolt (L48-57), 12 traces. corpus c3-scen-pikachu-ex-topaz-bolt
scenarios/pikachu-ex-tera-bench.json, ceruledge-ex-tera-bench.json - N's Darmanitan's Darman-i-cannon (forced bench target) hits a lone Tera Pokémon on the Bench: pikachu-ex.ts L67 / ceruledge-ex.ts L66 `preventDefault`, plus N's Darmanitan's bench branch. 12 traces each (some games end in the stuck prompt, see bugs). corpus c3-scen-pikachu-ex-tera-bench, c3-scen-ceruledge-ex-tera-bench
scenarios/iron-crown-ex-cobalt-command.json - iron-crown-ex.ts L68-75 (+20 for a Future attacker): Iron Treads attacks with Iron Crown ex on the Bench. 12 traces, covered. corpus c3-scen-iron-crown-ex-cobalt-command
scenarios/walking-wake-ex-special-condition.json - walking-wake-ex.ts shred path L45-60 and L71 (+120): opponent Poisoned. 12 traces covered. corpus c3-scen-walking-wake-ex-special-condition
scenarios/walking-wake-ex-ability-blocked.json - L55 `catch { return state }`: opposing Flutter Mane's Midnight Fluttering. 12 traces, covered. corpus c3-scen-walking-wake-ex-ability-blocked
scenarios/fezandipiti-dark-energy.json - fezandipiti.ts [D] Energy coin flip, heads/tails (16 traces, covered). scenarios/fezandipiti-ability-blocked.json - L46 return (Flutter Mane attacker locks it; the probe uses the attacker), 12 traces, covered.
scenarios/slowpoke-mep-86-confusion.json - Slowpoke MEP 86 Dopey Face prevent branch (was 2 games): Cofagrigus Perplex on Slowpoke; 12 traces, covered. corpus c3-scen-slowpoke-mep-86-confusion
scenarios/cofagrigus-wht-patrat.json - cofagrigus-pool.ts L68 `if (moveCheck.preventDefault) return` (Patrat in play): 12 traces, covered. scenarios/cofagrigus-wht-mist-energy.json - MoveCounters prevented by Mist Energy (parity only).
scenarios/galarian-zigzagoon-lick.json, antique-cover-fossil-power.json, antique-cover-fossil-protective-cover.json, shelgon-guard-press.json, torracat-flare-strike.json - support cards: 12 traces each, 0 diverged; manual trace check: Lick heads and tails with paralysis effect in 12/12 games, Perplex's AddSpecialConditions on the fossil in 12/12, fossil power used (51 ability actions). No coverage tool output for non-pool cards.

## Exemption candidates
- Rescue Board emergency-board.ts:24 and Sparkling Crystal sparkling-crystal.ts:29 (tool blocked): no ported card reduces ToolEffect (k::TOOL) and nothing sets `stadiumAndToolHaveNoEffectTurnsRemaining`.
- Canplay `supporterTurn > 0` returns (Carmine L26, Explorer's Guidance L24, Janine's L24, Drayton L83) and Carmine L29: oracle never calls canPlay for a Supporter after one was played (scenarios above).
- Iron Crown ex L48 `targets == null`: `selected || []` is never null.
- Scoop Up Cyclone L33 `: null`: prompt min 1, not cancellable.
- Cofagrigus WHT L53/L58 (empty selection of non-cancellable min-1 prompts), L62 (`damageToMove <= 0`: only damaged Pokémon are selectable and nothing changes damage between the prompts), L74 (`source.damage < 0`: damage moved equals source damage unless a card edits it).
- Fezandipiti L43 (`pokemonCard !== this`: nothing evolves from it; `phase !== ATTACK`: no ported card issues PutDamageEffect on it outside an attack) and L59 `effect.damage <= 0` (only via a ReduceDamage-weakened attacker, e.g. Sylveon ex's Magical Charm, whose attack then lands on a Fezandipiti with [D] on the following turn; needs a gust; not engineered).

## Changes outside engine/src/cards/impls/ (new files there are the ports)
- effects.rs: new effect kind 244 `Effect::MoveCounters { b, damage }` (MoveCountersAttackEffect: b.source = counters' source slot, b.target = recipient), type name MOVE_COUNTERS_EFFECT, `k::MOVE_COUNTERS`, atk_base/atk_base_mut. Only new kind; kinds 245-251 unused.
- Added `k::MOVE_COUNTERS` to every all-attack-effects list: Mist Energy, Rabsca, Shuppet (HIDE_N_SNEAK_KINDS length 34 -> 35, also feeds Banette/Poltchageist/Sinistcha/Antique Cover Fossil), Acerola's Mischief, Rock Fighting Energy, Empoleon ex, Milotic ex, Skeledirge. Patrat now also prevents MoveCounters (its TS does).
- state.rs/canonical.rs: `Player.ancient_supporter` (canonical `ancientSupporter`); `CardInst.attack_shred` (canonical `cards[...].attacks[i].shredAttack`). Great Tusk now reads `ancient_supporter` (its TS 3-more-cards branch).
- engine/CARD_PORTING.md note (per-card field writes, scenario state replay, Supporter canPlay).

## Twinleaf bugs
N's Darmanitan ASC 33 (N's Darmanitan JTG) (ns-darmanitan.ts:72) - opens the non-cancellable Choose-Pokémon prompt without checking for a Benched Pokémon, so the game gets stuck ("no valid answer") when the opponent has no Bench vs "also does 90 damage to 1 of your opponent's Benched Pokémon" (nothing happens without a Bench).
Walking Wake ex PRE 178 (Walking Wake ex TEF) (walking-wake-ex.ts:41-75) - the shred handler runs for every copy of the card anywhere and again after Cathartic Roar's own +120, so the extra 120 is re-applied by later copies and the ability is applied for any Active named Walking Wake ex; also writes `effect.attack.shredAttack` into the card state.
Walking Wake ex PRE 178 (walking-wake-ex.ts:75) - the +120 is added after the base damage was zeroed and dealt through the shred path, so it is dealt separately (with effects) vs one 240 attack.
Iron Crown ex PRE 158 (iron-crown-ex.ts:49-67) - Twin Shotels' own ApplyWeaknessEffect has its ignore flags unset and targets the opponent's Active even for a Benched pick, so Weakness/Resistance of the Active are applied vs "isn't affected by Weakness or Resistance".
Fezandipiti PRE 45 (fezandipiti.ts:34-40) - IS_ABILITY_BLOCKED and CheckProvidedEnergy use the attacker as `player` instead of the Fezandipiti's owner.
Antique Cover Fossil SCR (antique-cover-fossil.ts:78-100) - the "discard from play" ability is copied from Poké Doll: only works while Active (throws on the Bench) and puts the card on the bottom of the deck instead of the discard pile; no "can't be affected by Special Conditions" handling.
Ceruledge ex PRE 147 (Ceruledge exSAR PRE) (ceruledge-ex.ts:51-56) - Amethyst Rage pushes Energy straight to the discard pile and rebuilds `cards`, leaving the slot's `energies` list stale and firing no MoveCardsEffect.
Pikachu ex ASC 57 (pikachu-ex.ts:48) / Ceruledge ex - none else. Scoop Up Cyclone PRE 128 (scoop-up-cyclone.ts:28) - preventDefault with no later discard: the card is never discarded by itself (stays in hand).
Scramble Switch SSP 186 (scramble-switch.ts:20) - the card is moved to the Supporter area as an Item and never discarded by itself.
Carmine PRE 103: none.

## Oracle problems
- `cli.js state` ignores `header.scenario`; used a temp node script (not committed) passing `scenario` to GameRunner.
- A c3-c random game stuck on "no valid answer for Choose pokemon" after a TEF Rare Candy-style evolve prompt (not one of this batch's cards); and CANNOT_USE_POWER oracle error in c3-b game 100015 (other card). Both replayed identically in Rust.
- Support cards produce no coverage output (not in pool.json).

Branch: port-c3. Finish time:
2026-10-02T22:41:40Z
