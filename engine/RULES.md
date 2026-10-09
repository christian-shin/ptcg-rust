# Rules decisions

How the engine applies the rules where the card text alone doesn't settle it.
Judge rulings decide (`data/rulings/all.json`; `id` is the compendium id, `n`
its number), and a ruling's principle applies to every card with the same
pattern, not just the card it names. Each entry cites its ruling or the date
of the decision.

## Ability locks

### "Has no Abilities"

- Every Ability lock in the pool says the Pokémon "has no Abilities". While
  locked, the Ability doesn't exist: it can't be used, its continuous effects
  stop and its triggers don't fire. The engine keeps one "has no Ability" fact
  per Pokémon, and activation, continuous effects and triggers all read it.
- A lock whose own source has no Ability is off. Example: with Jellicent ex's
  Ability locked, Items can be played again.
- Damp (Psyduck, Golduck) removes only Abilities that would Knock Out their
  user.

### Precedence between locks (2026-10-08)

1. **Fixed precedence beats order.** Hide 'n' Sneak beats Midnight
   Fluttering (id2426 / n1877), and Luminous Wing beats Initialization
   (id2309 / n1786).
2. **One-way: the lock that turns the other off wins, whatever the order.**
   If lock A removes the Ability of B's source and B can't do the same to A,
   A applies. Flutter Mane as the opponent's Active Pokémon turns off Iron
   Thorns ex's Initialization, even when Iron Thorns ex was there first:
   Initialization never covers Flutter Mane (no Rule Box), so it can't
   prevent it.
3. **Mutual: the lock that took hold first wins** and prevents the other.
   When it turns off, the other takes hold at that moment (id142 / n135).
4. **Simultaneous: the turn player's lock takes hold first** (id140 / n133).
   The order is fixed at that moment and stays; it does not swap when the
   turn passes.

A lock **takes hold** when its conditions are first met while its source
has its Ability. Examples: becoming the Active Pokémon (Flutter Mane, Iron
Thorns ex, Jellicent ex, Team Rocket's Arbok), being on the Bench (Gastrodon),
getting a Tool (Genesect), the Stadium being played (Team Rocket's
Watchtower), or being released when the lock that prevented it turns off.

No two locks in the current pool can turn each other off, so rule 3 doesn't
yet decide a game. The engine implements it in full anyway, with take-hold
stamps, because more lock cards will come. The stamps live in the game state
(`CardInst::lock_stamp`) and are kept up to date as the board changes
(`lock_sync` in `spec/passive.rs`): a stamp is cleared when its lock goes off,
and simultaneous take-holds are stamped with the turn player's first.

### Locks on playing cards

- **A card in the hand is judged by its printed data.** In-play locks
  (Watchtower, Iron Thorns ex, Gastrodon, Flutter Mane) don't remove
  Abilities from cards in the hand. With Watchtower in play, a [C] Pokémon
  with an Ability still can't be played past Team Rocket's Arbok's Potent
  Glare (id2147 / n1662).
- **An action lock covers every way of doing that action with a card from the
  hand.** Bronzong's Evolution Jammer ("can't play any Pokémon from their hand
  to evolve") also stops Rare Candy. It doesn't stop an Evolution card that
  comes from somewhere else, such as Grand Tree's search of the deck (id1133
  / n1046, Archeops' Ancient Power, same wording).
- **"Can't play X from your hand" covers every way a card goes from the hand
  into play, including through an Ability or an attack.** Honchkrow GX's Ruler
  of the Night stops Charjabug's Battery and Porygon-Z's Crazy Code attaching a
  Special Energy from the hand (id25, id230); Rare Candy's Stage 2 is played
  from the hand (id285, id1998). So Genesect's ACE Nullifier also stops an
  Ability or attack attaching an ACE SPEC card from the hand (decided
  2026-10-09). The once-per-turn Energy attachment and the one Supporter per
  turn are separate limits that effects don't use up (APR C-09).
- Genesect's ACE Nullifier ("can't play any ACE SPEC cards from their hand")
  blocks only cards played from the hand. An Energy or Tool put onto a Pokémon
  from the deck or discard pile by an effect isn't played from the hand, and
  Enriching Energy and Telepathic Psychic Energy trigger only when attached
  from the hand.
- Locks are declared in the card specs as predicates over the card (`Pred`),
  not as card-category bitmasks.
  A lock (`BlockUse`) declares whom it binds (opponent / owner / both), the
  actions it stops (play Item / Supporter / Stadium, attach Tool / Energy from
  the hand, play a Pokémon, evolve from the hand, retreat, use the Stadium), a
  card predicate and an `except` predicate, and the source's conditions
  (Active, has a Tool, the card is the source itself). `play_locked` in
  `spec/passive.rs` is the one query: the passive's handler (execution) and
  legality both call it. A lock that lasts (an attack's "can't play Item cards
  next turn": Budew, Frillish, Galvantula ex, Scream Tail ex, Chi-Yu,
  Bronzong) is the same declaration (`LockDecl`: actions, card predicate,
  `except`, error code) stored on the locked player with the turns it has
  left; `play_locked` checks the in-play locks and then those
  (`lasting_locked`). There are no per-category flags.
- **Locks name rules actions, not engine paths.** Every way a Pokémon card
  goes from the hand into play (a Basic to the Bench, an Evolution played onto
  a Pokémon, Rare Candy's Stage 2) is `PlayPokemon`; evolving with a card from
  the hand is also `Evolve`. Each path asks for all the actions it is an
  instance of, so Team Rocket's Arbok declares only `PlayPokemon` (its text)
  and still stops Rare Candy; Bronzong declares `Evolve`.
- **One evolution routine.** Every evolution (from the hand, Rare Candy,
  Grand Tree, Salvatore, Dwebble, ...) runs one Evolve event carrying the
  card's source zone, then the same consequences. Locks and "when you play
  this Pokémon from your hand to evolve" triggers apply only when the source
  is the hand (id1133, id285, id1998).
- Jellicent ex's "Item cards or Pokémon Tool cards from their hand" stops a
  Tool attached from the hand only; a Tool put on by an effect from another
  zone isn't stopped. Team Rocket's Arbok's "any Pokémon that has an Ability
  from their hand" also stops a Fossil with an Ability (Antique Root Fossil),
  which is played as a Pokémon. It also stops Rare Candy into a Stage 2 with an Ability (except Team Rocket's): Rare Candy is still played from the hand (id1133 / n1046, id285, id1998), so Rare Candy offers no Stage 2 a play lock blocks, and can't be used when none is left.

## On-play Abilities (2026-10-08)

- An Ability that triggers when the card is played ("when you play this
  Pokémon from your hand onto your Bench", "... to evolve") triggers once
  the card has been played and is on the board. It is checked against the
  ordinary in-play locks at its slot. Dedechange "does not activate until
  Dedenne GX is on the Bench, at which time it is considered in play" and is
  blocked (id80 / n75). With Watchtower in play, Meowth ex has no Ability
  when it comes into play, so Last-Ditch Catch doesn't trigger (id2334 /
  n1808); the same holds with Iron Thorns ex in the Active Spot.
- So no lock needs a special case for cards in the hand: the hand check
  uses printed data, and the on-play trigger checks the card in play.

## Blocked effects (2026-10-08)

- An Ability or card whose effect is prevented can still be used or played;
  the effect is blocked and the use counts (once per turn is spent). A Pokémon
  that can't be affected by Special Conditions can still be the target of
  Hypnotoxic Laser (id290 / n275); Dusknoir's Cursed Blast still Knocks Out
  Dusknoir when Battle Cage blocks the counters (id2264 / n1765). So Hide 'n'
  Sneak and Festival Grounds block Volcanion ex's Scalding Steam without
  making it unusable. Only an Ability or card that can have no effect at all
  (an empty deck, nothing to heal) can't be used (ids 255, 925, 2362).
- Battle Cage stops counters from being placed on Benched Pokémon, so counters
  moved onto one leave their source and vanish (id2257 / n1758).

## Evolution timing (2026-10-09)

- Evolving with a card from the hand can't happen on the player's first turn,
  on a Pokémon put into play this turn, or twice in a turn (APR A-05). An
  effect that puts an Evolution card onto a Pokémon ignores these limits
  unless its text says otherwise (APR C-12): Grand Tree's text keeps both,
  Salvatore's allows "put into play this turn".
- A permission doesn't override a card's own restriction: Rare Candy can't be
  used during the first turn or on a Basic put into play this turn even with
  Forest of Vitality or Eevee's Boosted Evolution in effect (Rare Candy text,
  id1144, id1815).
- A permission applies only as its text says: Eevee ex's Rainbow DNA ("if you
  play it from your hand onto this Pokémon") doesn't apply to Grand Tree or
  Salvatore; Forest of Vitality lets a [G] Pokémon evolve only into a [G]
  Pokémon on the turn it was played.

## Putting onto the Bench (2026-10-09)

- Risky Ruins and similar "whenever … puts a Pokémon onto their Bench" effects
  apply to every way a Pokémon is put onto the Bench (hand, deck, discard
  pile; id2233), after the Pokémon is on the Bench, like other "enters play"
  effects. Moving a Pokémon to the Bench (retreat, switch) isn't putting it
  onto the Bench (id2329).

