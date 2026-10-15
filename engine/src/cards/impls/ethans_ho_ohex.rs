//! Ethan's Ho-Oh ex (ASC 26 / DRI 39): Golden Flame — once during your turn, you may attach up to 2 Basic [R] Energy cards from
//! your hand to 1 of your Benched Ethan's Pokémon. Shining Feathers — 160; heal 50 damage from each of your Pokémon.
//!
//! An Ability (once per turn): one Attach from the hand, 1 to 2 Basic Fire Energy cards onto one Benched Pokémon with the
//! Ethan's tag; at least 1 (JP FAQ audit: Golden Flame may attach just 1, min 1 max 2, so the usage record is never set with
//! nothing attached). Shining Feathers heals each of your Pokémon by its own Heal event after the damage (the heal is its
//! own step even when the damage was prevented).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "EthansHoOhEx",
    // Golden Flame: once during your turn, you may attach up to 2 Basic [R] Energy from your hand to
    // 1 of your Benched Ethan's Pokémon.
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn,
        needs: &[],
        steps: &[Step::new(Op::Attach(AttachSpec {
            from: ZoneRef(Who::Me, Zone::Hand),
            predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Fire Energy")]),
            slots: AttachSlots::Bench,
            target: SlotPred::Tag(crate::types::tag::ETHANS),
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
