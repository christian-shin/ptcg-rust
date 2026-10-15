//! Bloodmoon Ursaluna ex (TWM): Seasoned Skill — Blood Moon costs [C] less
//! for each Prize card your opponent has taken. Blood Moon — 240; during
//! your next turn, this Pokémon can't attack.
//!
//! Seasoned Skill is an `AttackCost` "less" on Blood Moon used by this Pokémon only (subject `Holder`; JP FAQ
//! Bloodmoon Ursaluna ex x Mew ex: a Mew ex using the Benched Ursaluna's Blood Moon through Memory Helix pays its full
//! cost; batch 8 C12b): 6 - prizesLeft [C] (none when the opponent has 6 or 0 prizes left). Blood Moon leaves "this
//! Pokémon can't attack during your next turn".
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "BloodmoonUrsalunaEx",
attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn)) }))] }],
    passives: &[Passive {
        origin: RuleSource::Ability,
        // Blood Moon costs [C] less for each Prize card your opponent has taken.
        modifier: Modifier::AttackCost(AttackCostSpec {
            change: CostChange::Reduce(Num::If(
                &Cond::All(&[Cond::Cmp(Num::PrizesLeft(Who::Opp), CmpOp::Ge, Num::Lit(1)), Cond::Cmp(Num::PrizesLeft(Who::Opp), CmpOp::Le, Num::Lit(5))]),
                &Num::Sub(&Num::Lit(6), &Num::PrizesLeft(Who::Opp)),
                &Num::Lit(0),
            )),
            attack: Some(0),
            subject: SlotPred::Holder,
            side: Side::Owner,
            ..AttackCostSpec::DEFAULT
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
