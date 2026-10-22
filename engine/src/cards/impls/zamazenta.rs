//! Zamazenta (DRI 146): Strong Bash - 70; during your opponent's next turn, if this Pokémon is damaged by an attack (even if this
//! Pokémon is Knocked Out), put damage counters on the Attacking Pokémon equal to the damage done to this Pokémon.
//!
//! Strong Bash arms the lasting trigger on this Pokémon with the amount "equal to the damage done" (`RetaliateAmount::DamageDone`):
//! in the E-03 window of the attack that damages it, the counters (the damage the Damage event did, after Weakness, Resistance and
//! every effect) go on the Attacking Pokémon wherever it is now, placed by this Pokémon's attack (a PlaceCounters event, not damage;
//! Bouffalant SSP's Ready to Ram).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Zamazenta",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Retaliate(RetaliateAmount::DamageDone) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
