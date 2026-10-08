//! Team Rocket's Venture Bomb (DRI): flip a coin. If heads, put 2 damage
//! counters on 1 of your opponent's Pokémon. If tails, put 2 damage counters
//! on your Active Pokémon.
//!
//! Twinleaf quirks kept: counters are a raw `damage += 20` (no effects).
//! Right after the coin prompt is created (before it resolves), a
//! MOVE_CARDS supporter→discard with no card list moves the WHOLE play pile
//! (this card plus anything else in it) to the discard.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsVentureBomb",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        // Flip a coin. If heads, put 2 damage counters on 1 of your opponent's Pokémon. If tails, put 2 damage counters on your Active Pokémon.
        steps: &[Step::new(Op::Coin(CoinSpec {
            heads: &[Step::new(Op::PlaceCounters(PlaceCountersSpec {
                target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Pokemon(Who::Opp), msg: "CHOOSE_POKEMON_TO_DAMAGE" }),
                counters: Num::Lit(2),
                cause: CounterCause::Effect,
            }))],
            tails: &[Step::new(Op::PlaceCounters(PlaceCountersSpec { target: SlotTarget::Slot(MY_ACTIVE), counters: Num::Lit(2), cause: CounterCause::Effect }))],
            ..CoinSpec::DEFAULT
        }))],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
