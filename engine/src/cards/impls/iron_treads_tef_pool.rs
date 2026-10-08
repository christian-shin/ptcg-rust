//! Iron Treads (TEF 118): Dual Core — as long as this Pokémon has a Future
//! Booster Energy Capsule attached, it is [F] and [M] type. Wheel Pass — 60;
//! move an Energy from this Pokémon to 1 of your Benched Pokémon.
//!
//! Twinleaf: Dual Core replaces `cardTypes` with [F, M] on a
//! CheckPokemonTypeEffect whose target's Pokémon is this card, when a tool
//! named "Future Booster Energy Capsule" is attached and the ability isn't
//! blocked. Wheel Pass (after the attack): nothing unless a Benched Pokémon
//! exists, this slot holds an Energy card and is the Active; then a
//! non-cancellable AttachEnergyPrompt (Active's cards → Bench, min 1 max 1)
//! whose callback MOVE_CARDS each transfer from the Active.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "IronTreadsTEFPool",
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
