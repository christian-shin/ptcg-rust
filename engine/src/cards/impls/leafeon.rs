//! Leafeon (TWM 11): Leaflet Blessings - attach a Basic [G] Energy card from your hand to 1 of your Benched Pokémon. If you
//! do, heal all damage from that Pokémon. Solar Beam - 70.
//!
//! An Attach event of the chosen card onto a Benched Pokémon (asked when a card and a Benched Pokémon exist, not
//! cancellable: Hydrapple ex's Ripening Charge with the attack's mandatory wording); `Cond::Done` (APR E-20) heals the
//! Pokémon attached to by all its damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Leafeon",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::after_damage(Op::Attach(AttachSpec {
                from: ZoneRef(Who::Me, Zone::Hand),
                predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Grass Energy")]),
                slots: AttachSlots::Bench,
                bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
                ..AttachSpec::DEFAULT
            })),
            Step::after_damage(Op::If(IfSpec {
                cond: Cond::Done,
                yes: &[Step::new(Op::Heal(HealSpec { target: SlotTarget::Slot(SlotExpr::Attached), hp: Num::DamageOn(SlotExpr::Attached), clear_conditions: false }))],
                no: &[],
            })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
