//! Team Rocket's Venture Bomb (DRI 179): flip a coin. If heads, put 2 damage
//! counters on 1 of your opponent's Pokémon. If tails, put 2 damage counters
//! on your Active Pokémon.
//!
//! Rule: heads or tails is one PlaceCounters event of 20 (cause: this Trainer card):
//! placing counters isn't damage, so no Weakness, Resistance or damage modifier
//! applies (APR C-07). It is a Trainer card's effect, so neither "prevent all effects
//! of attacks" (Mist Energy) nor Battle Cage (attacks and Abilities only) stops it. The
//! chosen Pokémon may be any of the opponent's, Benched or Active.
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
                counters: Num::Lit(2)
            }))],
            tails: &[Step::new(Op::PlaceCounters(PlaceCountersSpec { target: SlotTarget::Slot(MY_ACTIVE), counters: Num::Lit(2) }))],
            ..CoinSpec::DEFAULT
        }))],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
