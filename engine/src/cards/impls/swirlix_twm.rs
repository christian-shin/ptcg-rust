//! Swirlix (TWM 89): Festival Lead — if Festival Grounds is in play, this Pokémon may use an attack it has twice. If the first attack
//! Knocks Out your opponent's Active Pokémon, you may attack again after your opponent chooses a new Active Pokémon. Sneaky
//! Placement — put 2 damage counters on 1 of your opponent's Pokémon.
//!
//! Festival Lead is a declaration (`Modifier::AttackTwice`) the core reads when the second attack would be offered, for an attack
//! printed on this Pokémon (as Dipplin TWM 18). Sneaky Placement is one PlaceCounters event on the chosen Pokémon (chosen at step D),
//! counters not damage (APR C-07).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "SwirlixTWM",
    passives: &[Passive { origin: RuleSource::Ability, modifier: Modifier::AttackTwice(AttackTwiceSpec { guard: Cond::StadiumInPlay(Pred::Name("Festival Grounds")) }) }],
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::PlaceCounters(PlaceCountersSpec { target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Pokemon(Who::Opp) }), counters: Num::Lit(2) }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
