//! Mega Lucario ex (M1L): Aura Jab — 130. Attach up to 3 Basic [F] Energy
//! cards from your discard pile to your Benched Pokémon in any way you like.
//! Mega Brave — 270. During your next turn, this Pokémon can't use Mega Brave.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegaLucarioex",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::Attach(AttachSpec {
                from: ZoneRef(Who::Me, Zone::Discard),
                predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Fighting Energy")]),
                slots: AttachSlots::Bench,
                bounds: Bounds { min: Num::Lit(0), max: Num::Lit(3) },
                ..AttachSpec::DEFAULT
            }))],
        },
        AttackSpec { index: 1, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::CannotUseThisAttackNextTurn }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
