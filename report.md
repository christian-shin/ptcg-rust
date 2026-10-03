2026-10-03T03:14:42Z

# Re-verification batch r1 report

All 25 cards are pure reprints (the current class only extends a ported logic class and overrides set/number/fullName, plus regulationMark for Pokegear 3.0 SV11W). In every case the CardDef data equals the previously verified printing's, and the printing binds to the existing port (behavior class = previous printing's class). No port logic changed. Core changes: none. Only engine edit: pins on three ports whose class name is defined in several Twinleaf files (commit "pin Duskull, Dudunsparce, Lunatone ..."): `Duskull@Duskull SFA|Duskull PRE` (duskull.rs), `Dudunsparce@Dudunsparce TEF|Dudunsparce PRE` (dudunsparce.rs), `Lunatone@Lunatone M1L|Lunatone ASC` (lunatone_meg.rs). Goldeen was already pinned `Goldeen@Goldeen TWM|Goldeen PRE`. check_pins: ok.

Runs: parity r1-a (9 trainers) 16/16 pass, r1-b (9 Pokemon) 16/16, r1-c (7 Pokemon) 16/16, 0 diverged. Coverage (--remote 4): r1-a-cov, r1-b-cov, r1-c-cov 16/16 each, 0 diverged. Scenario runs: 12 traces each, 0 diverged (see entries). Oracle games ending `status=error CANNOT_USE_POWER` / `stuck` in the logs are Twinleaf-side and matched by Rust.

Chikorita key note: the batch lists "Chikorita MEP 69" (Twinleaf ChikoritaMEP, extends Chikorita from set-mega-evolution). The pool row was remapped to "Chikorita MEP 46" (class ChikoritaMEPPool, vanilla data card, no logic, behavior ""), so the key MEP 69 no longer exists in the pool; verified.json still has the stale key "Chikorita MEP 69" (needs-reverify) and should be replaced by Chikorita MEP 46.

## Cards

| Card | Case | Status | Notes |
|---|---|---|---|
| Air Balloon ASC 181 (Air Balloon ASC) | reprint | verified | tool-blocked L24 via scenario |
| Bloodmoon Ursaluna PRE 54 (Bloodmoon Ursaluna PRE) | reprint | partial-exempt | see exemptions: L42 and L56 unreachable; otherwise verified |
| Briar PRE 100 (Briar PRE) | reprint | partial | needs a Prize-count scenario edit (request below) |
| Chikorita MEP 46 (was listed MEP 69; Chikorita MEP) | vanilla data card | verified | parity + coverage, no logic |
| Crushing Hammer PBL 105 (Crushing Hammer FA PBL) | reprint | verified | L41 exempt (non-cancellable ChoosePokemonPrompt) |
| Cynthia's Spiritomb ASC 133 | reprint | verified | scenario L37 |
| Dudunsparce PRE 80 | reprint | verified | scenario empty deck L41 |
| Duskull PRE 35 | reprint | verified | scenarios; L23 exempt |
| Energy Switch PBL 107 (Energy Switch FA PBL) | reprint | verified | random coverage, 0 gaps |
| Fezandipiti ex ASC 142 | reprint | verified | scenarios L53/67/71/90-96 |
| Goldeen PRE 20 | reprint | verified | scenarios |
| Hydrapple ex PRE 11 | reprint | verified | Syrup Storm scenario |
| Jumbo Ice Cream CRI 109 (Jumbo Ice Cream FA CRI) | reprint | verified | L25 canPlay UI-only exempt |
| Lunatone ASC 105 | reprint | verified | scenarios |
| Mega Lucario ex ASC 113 | reprint | verified | Mega Brave scenario |
| N's Zoroark ex ASC 137 | reprint | verified | scenarios |
| Pecharunt SVP 129 (PecharuntIR SVP) | reprint | verified | scenario L61 catch |
| Pokégear 3.0 BLK 84 (Pokegear 3.0 SV11W) | reprint (regulationMark I only) | verified | empty-deck scenario; L55 canPlay exempt |
| Prism Energy ASC 216 | reprint | verified | random coverage, 0 gaps |
| Roaring Moon PRE 65 | reprint | verified | random coverage, 0 gaps |
| Team Rocket's Ariana ASC 202 | reprint | verified | scenario; L27 canPlay exempt |
| Team Rocket's Mewtwo ex ASC 79 | reprint | verified | scenarios; exemptions below |
| Team Rocket's Spidops ASC 19 | reprint | verified | L60 exempt |
| Terapagos ex ASC 179 | reprint | verified | Crown Opal scenario |
| Wondrous Patch POR 117 (Wondrous Patch FA POR) | reprint | verified | L88 exempt (AttachEnergyPrompt min 1, no cancel) |

## Exemptions
- Bloodmoon Ursaluna PRE 54: bloodmoon-ursaluna.ts L42 ability-blocked return (no ported lock blocks a Basic Fighting card in hand; Arbok throws first); L56 `cardList === undefined` (card is on the Bench when the callback runs).
- Duskull PRE 35: duskull.ts L23 `cards.length = slots.length` (prompt max is min(slots, 3)).
- Crushing Hammer: L41 `targets.length === 0` (allowCancel false). Wondrous Patch L88 `transfers.length === 0` (allowCancel false, min 1).
- Jumbo Ice Cream L25, Ariana L27, Pokegear L55, Briar L24: canPlay false returns, UI-only (oracle never calls canPlay with those boards).
- Mewtwo ex: L49 `? void 0` optional chaining; L62 `!hasBenched` (needs 4 Team Rocket's Pokemon with empty Bench, impossible); L66 `transfers === null` (allowCancel false). N's Zoroark ex L86/L87 optional chaining. Spidops L60 `cardList === undefined`.

## Scenario request (Briar PRE 100: partial)
Briar needs the opponent at exactly 2 Prize cards remaining; no scenario edit sets Prize counts. Request: a side edit such as `prizes_left: 2` (or `prizes_taken: N`). It would cover briar.ts L46 `this.extraPrizes = true` (1 game in random coverage), L54 `{ return state; }` and L60 `{ if (effect.prizeCount > 0) { effect.prizeCount += 1; } }` (both 0 games).

## Scenarios
All runs: 12 traces, 0 diverged. Corpora under corpus/cards/ (gitignored; cov/dump dirs deleted). Hydrapple/Mewtwo/Zoroark/Terapagos/Lucario/Dudunsparce scenarios were run with --remote 4 (local coverage records only ~2 games).

```
scenarios/air-balloon-tool-blocked.json - Air Balloon ASC 181 (Air Balloon ASC)
  targets:  air-balloon.ts:24 `{ return state; }` (IS_TOOL_BLOCKED)
  setup:    Active Petilil with Air Balloon, Bench Petilil, Jamming Tower in play
  result:   L24 >=3 games
  corpus:   corpus/cards/r1-scen-air-balloon-tool-blocked
scenarios/team-rockets-ariana-empty-deck.json - Team Rocket's Ariana ASC 202
  targets:  team-rockets-ariana.ts:63 `? 8`, :66 `{ break; }`
  setup:    only Team Rocket's Mewtwo ex in play, hand only Ariana, deck down to 1 card (51 Psychic Energy in discard)
  result:   both >=3 games
  corpus:   corpus/cards/r1-scen-team-rockets-ariana-empty-deck
scenarios/pokegear-30-empty-deck.json - Pokégear 3.0 BLK 84 (Pokegear 3.0 SV11W)
  targets:  pokegear-30.ts:18 `{ throw CANNOT_PLAY_THIS_CARD; }`
  setup:    52 Fighting Energy in discard so Prizes take the last deck cards; hand only Pokegear
  result:   L18 >=3 games
  corpus:   corpus/cards/r1-scen-pokegear-30-empty-deck
scenarios/bloodmoon-ursaluna-attach-energy.json - Bloodmoon Ursaluna PRE 54
  targets:  bloodmoon-ursaluna.ts:L58-64 attach callback (cards.length > 0), L61 MOVE_CARDS
  setup:    hand Ursaluna + 2 Fighting Energy, so the policy benches it and uses the ability
  result:   L61 >=3 games (below-3 list: only L42, L52, L56; L42/L56 exempt, L52 covered by random run)
  corpus:   corpus/cards/r1-scen-bloodmoon-ursaluna-attach
scenarios/cynthias-spiritomb-raging-curse-bench-damage.json - Cynthia's Spiritomb ASC 133
  targets:  cynthias-spiritomb.ts:L37 `{ totalDamage += pokemon.damage; }`
  setup:    Spiritomb Active, two damaged Cynthia's Gible on Bench, scripted Raging Curse
  result:   12 games L37
  corpus:   corpus/cards/r1-scen-cynthias-spiritomb
scenarios/duskull-come-and-get-you.json - Duskull PRE 35
  targets:  duskull.ts:L8 useKingsOrder body, L17-29, L61-63
  setup:    Duskull Active, Duskull in hand and 2 in discard, scripted Come and Get You
  result:   12 games each at L8, L17, L26, L63
  corpus:   corpus/cards/r1-scen-duskull-come-and-get-you
scenarios/duskull-come-and-get-you-no-hand-duskull.json - Duskull PRE 35
  targets:  duskull.ts:L14 `if (!hasDuskullInDiscard) { throw }`
  setup:    same board, no Duskull in hand, 3 in discard
  result:   12 games L14
  corpus:   corpus/cards/r1-scen-duskull-come-and-get-you-no-hand-duskull
scenarios/fezandipiti-ex-empty-deck-after-ko.json - Fezandipiti ex ASC 142
  targets:  fezandipiti-ex.ts:L52/53 empty-deck throw
  setup:    adapted from hassel-empty-deck-after-ko; deck emptied to 1 card
  result:   12 games L52
  corpus:   corpus/cards/r1-scen-fezandipiti-ex-empty-deck-after-ko
scenarios/fezandipiti-ex-cruel-arrow.json - Fezandipiti ex ASC 142
  targets:  fezandipiti-ex.ts:L90-96 Cruel Arrow prompt + 100 damage
  setup:    Fezandipiti Active with 3 Metal Energy, scripted Cruel Arrow, opponent Bench
  result:   12 games each at L90, L92, L93
  corpus:   corpus/cards/r1-scen-fezandipiti-ex-cruel-arrow
scenarios/fezandipiti-ex-poison-ko-between-turns.json - Fezandipiti ex ASC 142
  targets:  fezandipiti-ex.ts:L71 `{ return state; }`
  setup:    opponent Active at 200 damage and Poisoned, I pass
  result:   12 games L71
  corpus:   corpus/cards/r1-scen-fezandipiti-ex-poison-ko-between-turns
scenarios/fezandipiti-ex-confusion-self-ko.json - Fezandipiti ex ASC 142
  targets:  fezandipiti-ex.ts:L67 `{ return state; }`
  setup:    my Active Confused with 190 damage, coins [false], self-damage KO on my turn
  result:   12 games L67
  corpus:   corpus/cards/r1-scen-fezandipiti-ex-confusion-self-ko
scenarios/goldeen-pre-festival-grounds-discard.json - Goldeen PRE 20
  targets:  twilight-masquerade/goldeen.ts:54 `barrage = true`, :43/:45 discard callback, :39 return
  setup:    Goldeen with 2 Water Energy, Festival Grounds PRE in play, opponent Snorlax with 2 Water Energy, coins heads x4
  result:   12 games L54; 11 games L43 and L45; 7 games L39
  corpus:   corpus/cards/r1-scenB2-goldeen-pre-festival-grounds-discard
scenarios/goldeen-pre-no-stadium-discard.json - Goldeen PRE 20
  targets:  goldeen.ts:56 `else { barrage = false }`
  setup:    same without the stadium
  result:   12 games L56
  corpus:   corpus/cards/r1-scenB2-goldeen-pre-no-stadium-discard
scenarios/goldeen-pre-no-opponent-energy.json - Goldeen PRE 20
  targets:  goldeen.ts:39 `{ return state; }`
  setup:    no stadium, opponent Active without Energy
  result:   6 traces, 0 diverged (extra; L39 already 7 games in the festival run); corpus deleted
  corpus:   none
scenarios/lunatone-asc-lunar-cycle.json - Lunatone ASC 105
  targets:  lunatone.ts L39 used throw, L44 isSolrockInPlay, L60/L64 attach callback
  setup:    Lunatone Active, Solrock ASC Bench, 3 Fighting Energy in hand
  result:   10 games L39; 12 games L44, L60, L64
  corpus:   corpus/cards/r1-scenB2-lunatone-asc-lunar-cycle
scenarios/lunatone-asc-cancel-discard.json - Lunatone ASC 105
  targets:  lunatone.ts L62 `{ return; }` (cancelled prompt)
  setup:    as above with answers [ability, null]
  result:   12 games L62
  corpus:   corpus/cards/r1-scenB2-lunatone-asc-cancel-discard
scenarios/lunatone-asc-no-fighting-energy.json - Lunatone ASC 105
  targets:  lunatone.ts L57 throw (no Fighting Energy in hand)
  setup:    Solrock Benched, hand only Snorlax
  result:   12 games L57
  corpus:   corpus/cards/r1-scenB2-lunatone-asc-no-fighting-energy
scenarios/lunatone-asc-no-solrock.json - Lunatone ASC 105
  targets:  lunatone.ts L48 throw (no Solrock)
  setup:    Lunatone alone, 3 Fighting Energy in hand
  result:   12 games L48
  corpus:   corpus/cards/r1-scenB2-lunatone-asc-no-solrock
scenarios/lunatone-asc-empty-deck.json - Lunatone ASC 105
  targets:  lunatone.ts L50/51 empty-deck throw
  setup:    51 Fighting Energy in discard empties the deck after Prizes refill; Solrock Benched
  result:   12 games L50, code after (L53) 0 games
  corpus:   corpus/cards/r1-scenB2-lunatone-asc-empty-deck
scenarios/pecharunt-svp-ability-locked.json - Pecharunt SVP 129 (PecharuntIR SVP)
  targets:  pecharunt.ts L61 `catch (_a) { return state; }`
  setup:    opponent Active Pecharunt, me Active Flutter Mane PRE (strips the ability at the between-turns probe)
  result:   12 games L61 (L63 ran in 6)
  corpus:   corpus/cards/r1-scenB2-pecharunt-svp-ability-locked
scenarios/team-rockets-mewtwo-ex-erasure-ball.json - Team Rocket's Mewtwo ex ASC 79
  targets:  team-rockets-mewtwo-ex.ts L59-76 Erasure Ball DiscardEnergyPrompt, 160+60n
  setup:    Mewtwo with 3 Psychic Energy + 3 Team Rocket's Houndour DRI Bench with 5 Psychic Energy; scripted attack
  result:   below-3 list only L43 (other scenario), L49, L62 (exempt)
  corpus:   corpus/cards/r1-scen-team-rockets-mewtwo-ex-erasure-ball
scenarios/team-rockets-mewtwo-ex-iron-thorns-lock.json - Team Rocket's Mewtwo ex ASC 79
  targets:  team-rockets-mewtwo-ex.ts L43 `{ return state; }`
  setup:    opponent Active Iron Thorns ex PRE 32 locks Mewtwo; I attack
  result:   L43 >=3 games
  corpus:   corpus/cards/r1-scen-team-rockets-mewtwo-ex-iron-thorns-lock
scenarios/ns-zoroark-ex-night-joker-copy.json - N's Zoroark ex ASC 137
  targets:  ns-zoroark-ex.ts L80-96 Night Joker copy from Benched N's Pokemon
  setup:    N's Zorua -> Zoroark ex Active with 2 Darkness Energy, Bench N's Zorua with 2 Darkness; scripted Night Joker
  result:   only L86/L87 `? void 0` (exempt) below 3
  corpus:   corpus/cards/r1-scen-ns-zoroark-ex-night-joker-copy
scenarios/ns-zoroark-ex-night-joker-no-ns-bench.json - N's Zoroark ex ASC 137
  targets:  ns-zoroark-ex.ts L90-91 `nsPokemon.length === 0` return
  setup:    Bench holds only Hop's Snorlax
  result:   12 games L90, L91
  corpus:   corpus/cards/r1-scen-ns-zoroark-ex-night-joker-no-ns-bench
scenarios/ns-zoroark-ex-trade-empty-hand.json - N's Zoroark ex ASC 137
  targets:  ns-zoroark-ex.ts L57 CANNOT_USE_POWER (empty hand)
  setup:    Zoroark Active, empty hand
  result:   12 games L55, L57
  corpus:   corpus/cards/r1-scen-ns-zoroark-ex-trade-empty-hand
scenarios/hydrapple-ex-syrup-storm.json - Hydrapple ex PRE 11
  targets:  hydrapple-ex.ts L87-98 Syrup Storm damage by Grass/ANY Energy
  setup:    Applin -> Dipplin -> Hydrapple ex Active with 2 Grass, Bench Applin with Prism + Grass Energy; scripted attack
  result:   12 games L87, L95, L98
  corpus:   corpus/cards/r1-scen-hydrapple-ex-syrup-storm
scenarios/terapagos-ex-crown-opal.json - Terapagos ex ASC 179
  targets:  terapagos-ex.ts L41 CANNOT_USE_ATTACK throw, L47 PREVENT_DAMAGE
  setup:    Terapagos Active with Grass/Water/Lightning Energy, 2 Benched Snorlax; scripted Crown Opal at turn 2
  result:   0 segments below 3; 12 games L41, L47
  corpus:   corpus/cards/r1-scen-terapagos-ex-crown-opal
scenarios/mega-lucario-ex-mega-brave.json - Mega Lucario ex ASC 113
  targets:  mega-lucario-ex.ts L54 Mega Brave pending push
  setup:    Riolu ASC -> Mega Lucario ex ASC Active with 2 Fighting Energy; scripted Mega Brave
  result:   12 games L54
  corpus:   corpus/cards/r1-scen-mega-lucario-ex-mega-brave
scenarios/dudunsparce-empty-deck.json - Dudunsparce PRE 80
  targets:  dudunsparce.ts (TEF) L41 empty-deck throw
  setup:    Dunsparce -> Dudunsparce Active, 52 Psychic Energy in discard
  result:   12 games L39, L41
  corpus:   corpus/cards/r1-scen-dudunsparce-empty-deck
```

## Twinleaf bugs
```
Duskull PRE 35 (set-shrouded-fable/duskull.ts:11-15) - `hasDuskullInDiscard` inspects player.hand, so Come and Get You throws unless a Duskull is in hand vs "from your discard pile"
Duskull PRE 35 (set-shrouded-fable/duskull.ts:17-24) - with a full Bench max is 0 while min is 1, so the Choose cards prompt has no valid answer (stuck game)
```
Quirk (not a bug, ported as-is): Goldeen writes attacks[0].barrage at runtime; Briar's KnockOut handler moves the card from the KO'd player's supporter list.

Finish checks: diff over corpus/t1 + main corpora + worktree corpora: 34258 traces, 0 diverged (312 unsupported, 2 approved, one caught "prompt stack exhausted" panic on a stuck Duskull scenario game); cargo test --release ok; check_pins ok.
Finished: 2026-10-03T03:57:46Z
