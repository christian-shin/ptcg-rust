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
stamps, because more lock cards will come.

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
- Locks are declared in the card specs as predicates over the card (`Pred`),
  not as card-category bitmasks.

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

## Not yet implemented (2026-10-08)

- On-play triggers check locks while the card is still in the hand, so
  Watchtower doesn't stop Meowth ex (scenario
  `meowth-ex-last-ditch-catch-watchtower`, expected to fail until fixed).
- Iron Thorns ex's Initialization strips Abilities from hand cards, which lets
  a Rule Box Pokémon with an Ability be played past Potent Glare.
- Lock precedence gets rules 2 and 4 wrong: take-hold order is applied to
  one-way pairs too, ties go to whoever's turn it currently is, and stamps are
  set only when a Pokémon becomes Active.
