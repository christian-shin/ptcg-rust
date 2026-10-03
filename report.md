2026-10-03T08:26:19Z

# Batch p2 report (branch partials-p2)

Result: all 16 cards verified. 14 scenario files were added under `scenarios/` (commit 0b51712). No engine/Rust changes, no scenario-tool (`scenario.rs` / `scenario.ts`) changes were needed: every board could be built with the existing edits. No Twinleaf checkout changes.

Parity: `check_cards.py` runs p2-g1 (Rosa's Encouragement, Alakazam, Kadabra, Froslass, Mega Sharpedo ex, Team Rocket's Articuno, Blaziken ex, Victini; 16 traces, 0 diverged), p2-g2 (Ceruledge, Charmander, Mega Charizard X ex, Skeledirge, Dragonair, Mega Dragonite ex, Cofagrigus; 16 traces, 0 diverged), p2-annihilape (16 traces, 0 diverged; one game, seed 100005, ended with an oracle-side `CANNOT_USE_POWER` error when playing PRE-54#66 on turn 4 and is excluded by the tool; unrelated to Annihilape). Every scenario run below: 12 traces, 0 diverged.

Key facts used for the exemptions (all re-checked against today's pool):
* The pool's ability locks (Flutter Mane PRE 43, Gastrodon SSP 107, Iron Thorns ex PRE 32, Psyduck/Golduck, Team Rocket's Watchtower) are all remove-mode locks. `UsePowerEffect` runs `assertActivatedPowerNotLocked` (game-effect.ts:456-476, 484) with the real power's flags before the real `PowerEffect`; that probe equals the generic `IS_ABILITY_BLOCKED` probe for powers without lock flags, so a card's own `IS_ABILITY_BLOCKED` return inside `WAS_POWER_USED` can never be reached (the ability is stripped by CheckPokemonPowers and the use is rejected first).
* `EvolveEffect` is only created with the evolving card still in hand (play-pokemon-effect.ts:105, Rare Candy rare-candy.ts:154, Finizen). The lock callbacks of Gastrodon / Iron Thorns / Flutter Mane need the checked card on a Pokémon slot; Watchtower uses printed types of a card outside a slot ([C] only); Psyduck only hits `knocksOutSelf` powers.

## Cards

Rosa's Encouragement POR 84 (Rosa's Encouragement M3): verified
* rosas-encouragement.ts:26 canPlay `if (player.supporterTurn > 0) return false`: exempt. The oracle never evaluates canPlay with `supporterTurn > 0` (handbook: canPlay of Supporters is never reached with supporterTurn > 0); coverage p2-g1: 0 games, the neighbouring canPlay prize check runs in 16 games.
* rosas-encouragement.ts:102-105 attach loop: exempt. The prompt filter `{ superType: ENERGY, energyType: BASIC, stage: STAGE_2 }` is applied by `matchesPromptFilter` (prompt-card-filter.ts:36-71, used by oracle/options.ts:217,419 to build the selectable energies); no Energy card has a `stage` property (checked every `extends EnergyCard`), so no energy is ever selectable and min 0 leaves only `[]` (L83 `for (const transfer of transfers)` body: 0 games).

Annihilape PBL 41 (Annihilape M5): verified
* ability-blocked return (annihilape.ts:51) covered by annihilape-gastrodon-blocked (12 games: `12 games L46 { return state; }`).
* Durable Body coin flip, heads (annihilape.ts:59-64) covered by annihilape-durable-body (12 games: `12 games L53 { effect.surviveOnTenHPReason = ... }`).
* Ghostly Blow with benched targets covered by annihilape-ghostly-blow; empty-Bench return (annihilape.ts:73-75) covered by annihilape-ghostly-blow-no-bench (not listed below 3 games; the bench-prompt path runs in the other scenario).
* annihilape.ts:87-89 `if (!picked || picked.length === 0) return;`: exempt, ChoosePokemonPrompt min 1 allowCancel false (L67: 0 games).
* The earlier "no Mankey/Primeape" gap is gone: Mankey SSP 98 is a pool card and Primeape M5 a support card (data/support_cards.json); the scenarios build the Mankey/Primeape/Annihilape stack directly.

Alakazam MEG 56 (Alakazam M1S): verified
* alakazam.ts:42 ability-blocked return: exempt. JUST_EVOLVED runs with the card in hand (see key facts). It is a [P] Stage 2: Gastrodon / Flutter Mane / Iron Thorns (no Rule Box) need a slot; Watchtower hits [C] only; Psyduck needs knocksOutSelf. No `EvolveEffect` with the card already on a slot exists (Rare Candy and Play use the hand).

Kadabra MEG 55 (Kadabra M1S): verified
* kadabra.ts:41 ability-blocked return: exempt, same reason as Alakazam ([P] Stage 1 evolving from hand).

Froslass TWM 53: verified
* ability-blocked return (froslass.ts:48) covered by froslass-flutter-mane-blocked (12 games reach the block, `4 games L43 { return state; }`; opposing Flutter Mane in the Active Spot locks the Froslass Active).
* froslass.ts:98 `return state` after `if (state.phase === BETWEEN_TURNS)` (L76 `return state; }`, 0 games): exempt. `BetweenTurnsEffect` is only created in `runBetweenTurnsEffects` (game-phase-effect.ts:25), which `betweenTurns` (game-phase-effect.ts:37-58) only reaches after setting the phase to BETWEEN_TURNS (or when it already is, the only other caller path is `startNextTurn`, reached from EndTurn in PLAYER_TURN/ATTACK).

Mega Sharpedo ex PFL 61 (Mega Sharpedo ex M2): verified
* Greedy Fang draw (mega-sharpedo-ex.ts:34-41) covered by mega-sharpedo-greedy-fang (12 games, `12 games L43 MOVE_CARDS ...`), empty-deck return covered by mega-sharpedo-empty-deck (`12 games L39 { ... if (player.deck.cards.length === 0) { return state; }`, L43 0 games).
* mega-sharpedo-ex.ts:44-48 Hungry Jaws bonus block (L47, 0 games): exempt. It is a second `WAS_ATTACK_USED(effect, 0, this)` (attack index 0, Greedy Fang), and the first `WAS_ATTACK_USED(effect, 0, ...)` block always returns, so the second block is dead code.

Team Rocket's Articuno DRI 51: verified
* Repelling Veil (articuno.ts:37-59) covered by articuno-repelling-veil-counters (12 games: `12 games L59 { effect.preventDefault = true; }`; Alakazam M1S's Hand Power places counters on the Basic Team Rocket Articuno).
* articuno.ts:65-70 Dark Frost bonus (L66 `{ effect.damage += 60; }`, 0 games): exempt. It compares an Energy name with 'Team Rocket Energy'; the only such Energy card is named "Team Rocket's Energy" (team-rockets-energy.ts:22), no card is named 'Team Rocket Energy' (grep of the whole source tree: only articuno, zapdos and moltres ex compare against it).
* articuno.ts:53-54 `target.getPokemonCard()?.stage` / `?.hasTag` null side (L58, L59 `? void 0`, 0 games): exempt, the target of a PutCountersEffect is always an occupied Pokémon slot.

Blaziken ex JTG 24: verified
* blaziken-ex.ts:79-81 `if (transfers.length === 0) return;` (L65, 0 games): exempt, AttachEnergyPrompt min 1 max 1 allowCancel false (`validate` rejects length < min).

Victini SSP 21: verified
* ability-blocked return (victini.ts:47-48) covered by victini-flutter-mane-blocked (12 games: `12 games L45 return state;`; opposing Flutter Mane Active locks Victini's own Active when it uses Flare).
* victini.ts:65 `effect.source.getPokemonCard()?.evolvesFrom` null side (L58 `? void 0`, 0 games): exempt. The bonus only runs when `CheckPokemonTypeEffect(player.active)` contains [R] and the target is the opponent's Active; an empty source slot means the attacker left its slot during the attack (the only such attacks in the pool are Meowth ex Tuck Tail and Gholdengo Surf Back, whose attacker slot is the Active), and then `player.active` is empty, so `cardTypes` is `[]` (check-effects.ts:123-126) and the block is never entered. All other DealDamage producers (self-damage, Spiky Energy, copy attacks) have an occupied source or target the attacker.

Ceruledge PFL 20 (Ceruledge M2): verified
* ceruledge.ts:58-60 `else { effect.damage = 0; }` (not exactly 4 Fire Energy, L51, 0 games): exempt. ChooseCardsPrompt min 4 max 4 allowCancel false with filter name 'Fire Energy', so every answer holds exactly 4 matching cards (the oracle only offers selectable cards that match the filter).

Charmander PFL 11 (Charmander M2): verified
* not-top-Pokémon return (charmander.ts:34-36) covered by charmander-evolved-active-retreat (`12 games L39 { return state; }`: Charmander under Charmeleon M2 in the Active Spot, retreat cost checks).
* ability-blocked return (charmander.ts:39-41) covered by charmander-flutter-mane-blocked (`12 games L43 { return state; }`).

Mega Charizard X ex PFL 13 (Mega Charizard X ex M2): verified
* mega-charizard-x-ex.ts:75-77 `if (transfers === null) return;`: exempt. DiscardEnergyPrompt with allowCancel false and min 1: `validate(null)` returns `allowCancel` (false), so null is never an accepted answer. (The coverage tool reports no segment below 3 games for this file in p2-g2.)

Skeledirge SSP 31: verified
* `effect.target.getPokemonCard() !== this` return (skeledirge.ts:50-52, L49, 0 games): exempt. It needs Skeledirge in a slot below another Pokémon; nothing evolves from Skeledirge (no class names it in evolvesFrom / evolvesTo / evolvesToStage / evolvesFromBase anywhere in the source tree).
* ability-blocked `catch` return (skeledirge.ts:66-68) covered by skeledirge-gastrodon-blocked (`12 games L64 catch (_a) { return state; }`: benched Skeledirge under Gastrodon hit by Shadow Bullet's bench damage).
* Torcherto bench count (skeledirge.ts:37-42) covered by skeledirge-torcherto (12 games, `12 games L40 ? 1` and `: 0`).
* The all-attack-effects mask completed by b13: the prevent-effects branch (skeledirge.ts:46-83) ran in 12 games in the Gastrodon scenario (`12 games L46`); no new effect kinds were added.

Dragonair ASC 151 (Dragonair M2a): verified
* ability-blocked return (dragonair.ts:51-53, L56, 0 games): exempt, own-probe argument above (all pool locks are remove-mode locks that the `UsePowerEffect` probe already rejects).
* dragonair.ts:63-65 `if (!dragonairCardList) throw CANNOT_USE_POWER` (L66, 0 games): exempt, the ability can only be used from the top card of a slot of the player, so the card is always found.

Mega Dragonite ex ASC 152 (Mega Dragonite ex M2a): verified
* ability-blocked return (mega-dragonite-ex.ts:59-61, L60, 0 games): exempt, same reason as Dragonair (and Iron Thorns ex's lock is also remove-mode).
* no-Bench throw (L69) and Ryuno Glide energy discard (L84) covered by mega-dragonite-ex-no-bench-attack (12 games each).

Cofagrigus WHT 40: verified
* cofagrigus-pool.ts:52-54 `!selected || selected.length === 0` (L53) and :57-59 (L58): exempt, ChoosePokemonPrompt min 1 max 1 allowCancel false.
* :68-70 `if (damageToMove <= 0) return;` (L62): exempt, the source prompt blocks every Pokémon with `damage === 0` and nothing changes damage between the two prompts.
* :83-85 clamp `if (moveEffect.source.damage < 0)` (L74): exempt, `moveEffect.damage` equals the source's damage; the only handlers of MoveCountersAttackEffect in the pool (Patrat) only set `preventDefault`, so damage is never above `source.damage`.
* MoveDamageCounters prevention (:75-77) was covered by the c3 scenarios cofagrigus-wht-patrat / cofagrigus-wht-mist-energy (unchanged).

## Scenarios
scenarios/annihilape-ghostly-blow.json - Annihilape PBL 41 (Annihilape M5)
  targets:  sets/11-mega-evolution/set-pitch-black/annihilape.ts:73-95 Ghostly Blow with an opposing Bench (bench prompt + PlaceDamageCountersEffect)
  setup:    Mankey SSP 98 / Primeape M5 / Annihilape stack Active with 2 Psychic Energy, opponent Hop's Snorlax Active + Bench; answers: attack Ghostly Blow
  result:   12 traces, 0 diverged; attack block not listed below 3 games (L60-L73 run in >= 3 games)
  corpus:   corpus/cards/p2-scen-annihilape-ghostly-blow
scenarios/annihilape-ghostly-blow-no-bench.json - Annihilape PBL 41 (Annihilape M5)
  targets:  annihilape.ts:73 `if (!opponent.bench.some(...)) return state;`
  setup:    same, opponent has no Bench
  result:   12 traces, 0 diverged; L63 `{ return state; }` not listed below 3 games
  corpus:   corpus/cards/p2-scen-annihilape-ghostly-blow-no-bench
scenarios/annihilape-durable-body.json - Annihilape PBL 41 (Annihilape M5)
  targets:  annihilape.ts:59-66 Durable Body coin flip, heads
  setup:    opponent Active Annihilape stack hit by Marnie's Grimmsnarl ex ASC 287 Shadow Bullet (180 x2 Weakness >= 150 HP); coins: [true]
  result:   12 traces, 0 diverged; `12 games L53 { effect.surviveOnTenHPReason = this.powers[0].name; }`
  corpus:   corpus/cards/p2-scen-annihilape-durable-body
scenarios/annihilape-gastrodon-blocked.json - Annihilape PBL 41 (Annihilape M5)
  targets:  annihilape.ts:51-53 `if (IS_ABILITY_BLOCKED(...)) { return state; }`
  setup:    Gastrodon SSP 107 on my Bench (locks Benched Stage 2 of both players), Annihilape stack on opponent's Bench, Shadow Bullet's 30 Bench damage puts a PutDamageEffect on it
  result:   12 traces, 0 diverged; `12 games L46 { return state; }`
  corpus:   corpus/cards/p2-scen-annihilape-gastrodon-blocked
scenarios/froslass-flutter-mane-blocked.json - Froslass TWM 53
  targets:  sets/10-scarlet-and-violet/set-twilight-masquerade/froslass.ts:48-50 `if (IS_ABILITY_BLOCKED(...)) { return state; }`
  setup:    Snorunt TWM 51 / Froslass TWM 53 Active with 2 Water Energy, opponent Flutter Mane PRE 43 Active (locks the opposing Active's Abilities); the Chilling Curtain marker is set at end of turn
  result:   12 traces, 0 diverged; `4 games L43 { return state; }`
  corpus:   corpus/cards/p2-scen-froslass-flutter-mane-blocked
scenarios/victini-flutter-mane-blocked.json - Victini SSP 21
  targets:  sets/10-scarlet-and-violet/set-surging-sparks/victini.ts:47-48 `if (IS_ABILITY_BLOCKED(store, state, player, this)) return state;`
  setup:    Victini Active with Fire Energy attacks with Flare into an opposing Flutter Mane PRE 43 Active
  result:   12 traces, 0 diverged; `12 games L45 return state;` (code after it: 0 games)
  corpus:   corpus/cards/p2-scen-victini-flutter-mane-blocked
scenarios/charmander-evolved-active-retreat.json - Charmander PFL 11 (Charmander M2)
  targets:  sets/11-mega-evolution/set-phantasmal-flames/charmander.ts:34-36 `if (pokemonCard !== this) return state;`
  setup:    Charmander M2 under Charmeleon M2 in the Active Spot with a Benched Pokémon: every retreat cost check reaches the Charmander under the top card
  result:   12 traces, 0 diverged; `12 games L39 { return state; }`
  corpus:   corpus/cards/p2-scen-charmander-evolved-active-retreat
scenarios/charmander-flutter-mane-blocked.json - Charmander PFL 11 (Charmander M2)
  targets:  charmander.ts:39-41 `if (IS_ABILITY_BLOCKED(...)) return state;`
  setup:    Basic Charmander Active with a Bench, opponent Flutter Mane PRE 43 Active
  result:   12 traces, 0 diverged; `12 games L43 { return state; }`
  corpus:   corpus/cards/p2-scen-charmander-flutter-mane-blocked
scenarios/mega-sharpedo-greedy-fang.json - Mega Sharpedo ex PFL 61 (Mega Sharpedo ex M2)
  targets:  sets/11-mega-evolution/set-phantasmal-flames/mega-sharpedo-ex.ts:34-42 Greedy Fang draw
  setup:    Carvanha PFL 60 / Mega Sharpedo ex stack with 1 Darkness Energy (only Greedy Fang is payable); answers: attack Greedy Fang
  result:   12 traces, 0 diverged; `12 games L43 prefabs_1.MOVE_CARDS(...); return state; }`
  corpus:   corpus/cards/p2-scen-mega-sharpedo-greedy-fang
scenarios/mega-sharpedo-empty-deck.json - Mega Sharpedo ex PFL 61 (Mega Sharpedo ex M2)
  targets:  mega-sharpedo-ex.ts:37-39 `if (player.deck.cards.length === 0) { return state; }`
  setup:    reset; all other cards of my deck go to the discard pile except the 6 Prize cards, so the deck is empty at the attack
  result:   12 traces, 0 diverged; `12 games L39 { ... if (player.deck.cards.length === 0) { return state; }` (L43: 0 games)
  corpus:   corpus/cards/p2-scen-mega-sharpedo-empty-deck
scenarios/articuno-repelling-veil-counters.json - Team Rocket's Articuno DRI 51
  targets:  sets/10-scarlet-and-violet/set-destined-rivals/team-rockets-articuno.ts:37-59 PutCountersEffect prevention
  setup:    Articuno Active (opponent); my Abra MEG 54 / Kadabra / Alakazam MEG 56 stack uses Hand Power, which places counters through a PutCountersEffect in the attack phase
  result:   12 traces, 0 diverged; `12 games L59 { effect.preventDefault = true; }`
  corpus:   corpus/cards/p2-scen-articuno-repelling-veil-counters
scenarios/skeledirge-torcherto.json - Skeledirge SSP 31
  targets:  sets/10-scarlet-and-violet/set-surging-sparks/skeledirge.ts:37-42 Torcherto Bench count (occupied and empty slots on both sides)
  setup:    Fuecoco SSP 29 / Crocalor SVP / Skeledirge stack Active with 2 Fire Energy and one Benched Pokémon; opponent Active + 1 Benched; answers: attack Torcherto
  result:   12 traces, 0 diverged; `12 games L40 ? 1`, `12 games L40 : 0`, `12 games L41 ? 1`
  corpus:   corpus/cards/p2-scen-skeledirge-torcherto
scenarios/skeledirge-gastrodon-blocked.json - Skeledirge SSP 31
  targets:  skeledirge.ts:58-67 `catch { return state; }` around the ability probe
  setup:    Skeledirge stack on opponent's Bench, Gastrodon SSP 107 on my Bench, Marnie's Grimmsnarl ex Shadow Bullet's 30 Bench damage hits Skeledirge
  result:   12 traces, 0 diverged; `12 games L64 catch (_a) { return state; }`
  corpus:   corpus/cards/p2-scen-skeledirge-gastrodon-blocked
scenarios/mega-dragonite-ex-no-bench-attack.json - Mega Dragonite ex ASC 152 (Mega Dragonite ex M2a)
  targets:  sets/11-mega-evolution/set-ascended-heroes/mega-dragonite-ex.ts:69-72 no-Bench throw and :89-91 Ryuno Glide energy discard
  setup:    Dratini M2a / Dragonair / Mega Dragonite ex stack Active with Water + 2 Lightning Energy and no Bench; answers: attack Ryuno Glide
  result:   12 traces, 0 diverged; `12 games L68 ... if (!hasBenched) { throw ... CANNOT_USE_POWER }`, `12 games L84 { costs_1.DISCARD_X_ENERGY_FROM_THIS_POKEMON(...); }`
  corpus:   corpus/cards/p2-scen-mega-dragonite-ex-no-bench-attack

## Scenario-tool changes
None (scenario.rs / scenario.ts untouched; no edits requested by p1).

## Other changes outside impls/
None. No engine, data, or card-port changes.

## Twinleaf bugs seen
Mega Sharpedo ex PFL 61 (Mega Sharpedo ex M2) (mega-sharpedo-ex.ts:44) - Hungry Jaws' +150 block checks `WAS_ATTACK_USED(effect, 0, this)` (Greedy Fang) after a first block that always returns, so the bonus never applies vs "+150 if this Pokémon has damage counters" (already noted in the batch table; Greedy Fang also draws 1 card, not 2).
Team Rocket's Articuno DRI 51 (team-rockets-articuno.ts:72) - checks the name 'Team Rocket Energy', so the +60 never applies vs "Team Rocket's Energy".
Rosa's Encouragement POR 84 (Rosa's Encouragement M3) (rosas-encouragement.ts:98) - the AttachEnergyPrompt filter includes `stage: Stage.STAGE_2`, which no Energy card has, so no Energy can ever be selected and the card attaches nothing vs "attach up to 2 Basic Energy to 1 of your Stage 2 Pokémon".

## Final checks
* `engine/target/iter/diff /Users/christianshin/Documents/pkmntcg/corpus/t1 /Users/christianshin/Documents/pkmntcg/corpus/cards/*/ corpus/cards/*/ --quiet`: 37150 traces, 0 diverged, 312 unsupported, 2 approved (one "prompt stack exhausted" panic message at src/game.rs:384 comes from an older main-checkout corpus, no engine code changed on this branch; the p2 corpora alone: 216 traces, 0 diverged, 0 unsupported).
* `cargo test --release`: all test results ok.
* `python3 tools/check_pins.py engine/src/cards/impls`: pins ok.

Finished: 2026-10-03T09:22:26Z
