//! Blaziken ex (JTG): Seething Spirit — once during your turn, attach a Basic Energy card from your discard pile to 1 of your
//! Pokémon. Burning Assault — 200; during your next turn, this Pokémon can't attack.
//!
//! Seething Spirit is an Ability used once per turn (the marker OVERFLOWING_SPIRIT_MARKER is the usage record, cleared with the
//! turn's end): exactly 1 Basic Energy card from the discard pile, attached by an Attach event with the Ability as its cause.
//! Burning Assault arms a lasting `CANT_ATTACK` lock on this Pokémon until the end of your next turn.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Blazikenex",
    // Seething Spirit: once during your turn, attach a Basic Energy card from your discard pile to
    // 1 of your Pokémon.
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn("OVERFLOWING_SPIRIT_MARKER"),
        needs: &[],
        steps: &[Step::new(Op::Attach(AttachSpec {
            from: ZoneRef(Who::Me, Zone::Discard),
            slots: AttachSlots::BenchActive,
            bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
            ..AttachSpec::DEFAULT
        }))],
    }],
    // Burning Assault: during your next turn, this Pokémon can't attack.
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn)) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
