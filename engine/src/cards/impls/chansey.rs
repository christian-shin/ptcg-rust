//! Chansey (TWM): Lucky Attachment — attach a Basic Energy card from your
//! hand to 1 of your Pokémon. Boundless Power — 80; during your next turn,
//! this Pokémon can't attack.
//!
//! Lucky Attachment does nothing with no Basic Energy in hand (fixed in
//! R1-13: it used to throw CANNOT_USE_ATTACK, so the attack was not offered).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Chansey",
    attacks: &[
        // Lucky Attachment: attach a Basic Energy card from your hand to 1 of your Pokémon (does
        // nothing with no Basic Energy in hand).
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::If(IfSpec {
                cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Hand), Pred::BasicEnergy),
                yes: &[Step::new(Op::Attach(AttachSpec {
                    from: ZoneRef(Who::Me, Zone::Hand),
                    slots: AttachSlots::ActiveBench,
                    bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
                    ..AttachSpec::DEFAULT
                }))],
                no: &[],
            }))],
        },
        // Boundless Power: during your next turn, this Pokémon can't attack.
        AttackSpec { index: 1, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn)) }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
