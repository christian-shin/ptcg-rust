//! Iron Treads (TEF 118): Dual Core - as long as this Pokémon has a Future Booster Energy Capsule attached, it is [F] and [M]
//! type. Wheel Pass - 60; move an Energy from this Pokémon to 1 of your Benched Pokémon.
//!
//! Dual Core is a `TypeOverride` on this Pokémon (`IsThisPokemon`) with a Tool of that name attached, while its Ability works
//! (and the Tool isn't off); the derived type read (`derived::types`) sees it, so every "[F] Pokémon" / "[M] Pokémon" read of the
//! game does. Wheel Pass moves one Energy from this Pokémon (the Active) to a Benched Pokémon after the damage (a MoveEnergy event,
//! id1653): nothing without a Benched Pokémon or an Energy on it, no cancel.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "IronTreads",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(Op::If(IfSpec { cond: Cond::All(&[Cond::AnySlot(SlotSel::Bench(Who::Me), SlotPred::Any), Cond::AnySlot(SlotSel::One(SlotExpr::This), SlotPred::All(&[SlotPred::IsActive, SlotPred::HasCard(Pred::Energy)]))]), yes: &[Step::new(Op::Attach(AttachSpec { from: ZoneRef(Who::Me, Zone::Attached(SlotExpr::Active(Who::Me))), predicate: Pred::Energy, bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, ..AttachSpec::DEFAULT }))], no: &[] })),
        ] },
    ],
    passives: &[
        Passive { origin: RuleSource::Ability, modifier: Modifier::TypeOverride(TypeOverrideSpec { subject: SlotPred::All(&[SlotPred::IsThisPokemon, SlotPred::AnyTool(Pred::Name("Future Booster Energy Capsule"))]), set: &[ct::FIGHTING, ct::METAL] }) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
