//! Cofagrigus (WHT 40): Extended Damagriiigus — move all damage counters from
//! 1 of your Benched Pokémon to 1 of your opponent's Pokémon. Perplex — 60;
//! your opponent's Active Pokémon is now Confused.
//!
//! Twinleaf: Extended Damagriiigus does nothing without a damaged Benched
//! Pokémon. Otherwise a non-cancellable ChoosePokemonPrompt over the Bench
//! (the Active and undamaged Pokémon blocked), then a second one over the
//! opponent's Active and Bench. The callback reads the source's damage after
//! the second prompt; with none left it stops, else a MoveDamageCountersEffect
//! (preventable) and a MoveCountersAttackEffect (source = the Benched
//! Pokémon, target = the chosen Pokémon) are reduced; the counters always
//! leave the source, and reach the target unless the effect was prevented.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "CofagrigusWHTPool",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::MoveCounters(MoveCountersSpec { kind: MoveCountersKind::AllFromOne { from: PickSlotSpec { chooser: Who::Me, among: SlotSel::Filtered(&SlotSel::Bench(Who::Me), SlotPred::Damaged), msg: "CHOOSE_POKEMON" }, to: PickSlotSpec { chooser: Who::Me, among: SlotSel::Pokemon(Who::Opp), msg: "CHOOSE_POKEMON_TO_DAMAGE" } } })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(inflict(&[SpecialCondition::Confused], Cause::Attack)),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
