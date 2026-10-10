//! Ethan's Ho-Oh ex (DRI / ASC): Golden Flame — once during your turn, you
//! may attach up to 2 Basic [R] Energy from your hand to 1 of your Benched
//! Ethan's Pokémon. Shining Feather — 160; heal 50 damage from each of your
//! Pokémon.
//!
//! Twinleaf quirks kept: the marker check throws BLOCKED_BY_EFFECT; the
//! prompt filter is `name: 'Fire Energy'`, blockedTo lists every non-Ethan's
//! Pokémon; the marker and the ability animation happen when the prompt
//! resolves; each Energy is its own MOVE_CARDS (no AttachEnergyEffect).
//! Fixed (phase 4b #42): the power also throws CANNOT_USE_POWER without a
//! Benched Ethan's Pokémon, and the prompt is min 1 (it was min 0, so the
//! once-per-turn marker could be set with nothing attached, and with no
//! Benched Ethan's Pokémon the prompt had no valid target). The attack heals each of
//! your Pokémon with a HealEffect.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "EthansHoOhex",
    // Golden Flame: once during your turn, you may attach up to 2 Basic [R] Energy from your hand to
    // 1 of your Benched Ethan's Pokémon.
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn("SHINING_FEATHER_MARKER"),
        needs: &[],
        steps: &[Step::new(Op::Attach(AttachSpec {
            from: ZoneRef(Who::Me, Zone::Hand),
            predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Fire Energy")]),
            slots: AttachSlots::Bench,
            target: Pred::Tag(crate::types::tag::ETHANS),
            bounds: Bounds { min: Num::Lit(1), max: Num::Lit(2) },
            same_target: true,
            ..AttachSpec::DEFAULT
        }))],
    }],
    // Shining Feather: heal 50 damage from each of your Pokémon.
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::ForEach(ForEachSpec {
            over: SlotSel::Pokemon(Who::Me),
            body: &[Step::new(Op::Heal(HealSpec { target: SlotTarget::Slot(SlotExpr::Picked), hp: Num::Lit(50), clear_conditions: false }))],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
