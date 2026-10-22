//! N's Joltik (JTG 49): Zapping Short - 30; before doing damage, discard all Pokémon Tools from your opponent's Active Pokémon.
//! If you discarded a Pokémon Tool in this way, your opponent's Active Pokémon is now Paralyzed.
//!
//! The Tools leave play before the damage (Vullaby's Pluck); "if you discarded a Pokémon Tool in this way" is `Cond::Done` (APR E-20: the
//! Discard did at least part of it; with no Tool, or every Tool kept by a protection, nothing was discarded). The Paralysis is
//! part of the same "before doing damage" text, so it is a GainCondition before the damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "NsJoltik",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::before_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Opp, Zone::Tools(OPP_ACTIVE)), ..DiscardSpec::DEFAULT })),
            Step::before_damage(Op::If(IfSpec { cond: Cond::Done, yes: &[Step::new(inflict(&[SpecialCondition::Paralyzed]))], no: &[] })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
