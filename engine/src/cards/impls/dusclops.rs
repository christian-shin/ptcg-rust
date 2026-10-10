//! Dusclops (SFA): Cursed Blast — once during your turn, you may put 5 damage
//! counters on 1 of your opponent's Pokémon. If you placed any damage counters in
//! this way, this Pokémon is Knocked Out. Will-o-Wisp — 50.
//!
//! The counters are a PlaceCounters event with the Ability as its cause (not damage, APR C-07). When Hide 'n' Sneak or
//! Battle Cage refuses them nothing is placed and this Pokémon is still Knocked Out (id2264, id2425, JP FAQ Dusclops PRE
//! 36 / Battle Cage with Dusknoir). `Op::KnockOut` on this Pokémon is a KnockOut by an effect: it waits for the state
//! check with every other Knock Out (D1) and the opponent takes the Prize. No once-per-turn marker: the card leaves play.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Dusclops",
    powers: &[PowerSpec {
        index: 0,
        once: Once::No,
        needs: &[],
        steps: &[
            Step::new(Op::PlaceCounters(PlaceCountersSpec { target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::PokemonBenchFirst(Who::Opp), msg: "CHOOSE_POKEMON_TO_DAMAGE" }), counters: Num::Lit(5) })),
            Step::new(Op::KnockOut(KnockOutSpec { target: SlotExpr::This, when: Cond::True })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
