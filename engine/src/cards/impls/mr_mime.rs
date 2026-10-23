//! Mr. Mime (TEF 63): Look-Alike Show - your opponent reveals their hand; you may use the effect of a Supporter card you find
//! there as the effect of this attack. Eerie Wave - 20; your opponent's Active Pokémon is now Confused.
//!
//! Look-Alike Show is `Op::UseTrainer` over the opponent's hand (events batch 7, user decision D5): the Supporter's effect is
//! used, not played (no play rule: id2225, id2226; 0 or 1, id2376), its printed conditions still apply (official JP FAQ, Fire
//! Energy MEE 2 / Mela), and it is the attack's effect done to whatever it affects (id2025). A Supporter whose effect would do
//! nothing can be chosen.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MrMime",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::after_damage(Op::UseTrainer(UseTrainerSpec { from: ZoneRef(Who::Opp, Zone::Hand), kind: PlayKind::Supporter, required: false }))] },
        AttackSpec { index: 1, steps: &[Step::after_damage(inflict(&[SpecialCondition::Confused]))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
