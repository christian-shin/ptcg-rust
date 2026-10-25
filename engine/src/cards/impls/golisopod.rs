//! Golisopod (CRI 26): Critical Slash - 30; if your opponent's Pokémon is Knocked Out by damage from this attack, during your
//! opponent's next turn, prevent all damage from and effects of attacks done to this Pokémon. Boundless Power - 150; during your next
//! turn, this Pokémon can't use attacks.
//!
//! Critical Slash: after the damage, when the Defending Pokémon has no HP left (it is Knocked Out by this attack's damage at the state
//! check that follows; a Pokémon that survives, by a survive-on-10 effect for one, keeps the condition false) the lasting
//! prevention of damage and of the effects of attacks is armed on this Pokémon through the opponent's next turn (Hop's Phantump's
//! Splashing Dodge). Boundless Power arms the lasting lock on this Pokémon's attacks through your next turn (APR C-15).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Golisopod",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::If(IfSpec {
                cond: Cond::Slot(OPP_ACTIVE, SlotPred::RemainingHpAtMost(0)),
                yes: &[Step::new(Op::Arm(ArmSpec { what: Lasting::PreventDamage(DamageSource::Any) })), Step::new(Op::Arm(ArmSpec { what: Lasting::PreventAttackEffects }))],
                no: &[],
            }))],
        },
        AttackSpec { index: 1, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn)) }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
