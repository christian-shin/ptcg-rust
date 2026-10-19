//! Hydrapple ex (SCR / PRE 11): Ripening Charge - once during your turn, you may attach a Basic [G] Energy card from your
//! hand to 1 of your Pokémon. If you attached Energy to a Pokémon in this way, heal 30 damage from that Pokémon. Syrup Storm
//! - 30+; 30 more damage for each [G] Energy attached to all of your Pokémon.
//!
//! Ripening Charge is an Attach event of the chosen card (the prompt can be cancelled); "If you attached Energy ... in this
//! way" is `Cond::Done` on it (APR E-20): the heal is one Heal event on the Pokémon attached to. The Ability's once-per-turn
//! usage record (ending with the owner's turn) is made when it is used, whether or not an Energy is attached (id127,
//! id249: an Ability that did nothing is still used). Syrup
//! Storm counts the [G] Energy provided by every Pokémon of yours (an Energy that provides [G] or any type counts).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "HydrappleEx",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Mul(&Num::EnergyOn(SlotSel::Pokemon(Who::Me), EnergyUnit::Provided(ct::GRASS)), &Num::Lit(30)), when: Cond::True })),
        ] },
    ],
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn,
        needs: &[],
        steps: &[
            Step::new(Op::Attach(AttachSpec { from: ZoneRef(Who::Me, Zone::Hand), predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Grass Energy")]), slots: AttachSlots::ActiveBench, bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, cancel: true, ..AttachSpec::DEFAULT })),
            Step::new(Op::If(IfSpec { cond: Cond::Done, yes: &[Step::new(Op::Heal(HealSpec { target: SlotTarget::Slot(SlotExpr::Attached), hp: Num::Lit(30), clear_conditions: false }))], no: &[] })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
