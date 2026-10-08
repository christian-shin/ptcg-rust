//! Blaziken ex (JTG): Seething Spirit — once during your turn, attach a Basic
//! Energy card from your discard pile to 1 of your Pokémon. Burning Assault
//! — 200; during your next turn, this Pokémon can't attack.
//!
//! Twinleaf: the marker (OVERFLOWING_SPIRIT_MARKER) and the ABILITY_USED
//! board effect are set in the prompt callback; the marker is removed on any
//! EndTurnEffect for the ending player.
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
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::CannotAttackNextTurn }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
