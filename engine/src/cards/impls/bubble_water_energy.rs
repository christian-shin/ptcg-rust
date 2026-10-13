//! Bubbly Water Energy (CRI, class key BubbleWaterEnergy): provides [W]. The [W] Pokémon this card is attached to recovers from
//! all Special Conditions and can't be affected by any Special Conditions.
//!
//! Two Energy-origin declarations over the Holder while it is [W] as the game checks its type (`TypeIs`): a `Recover` of all
//! conditions at the table check and a `Prevent` over GainCondition whatever the cause (an attack, an Ability or a Trainer;
//! events batch 4). A Pokémon of another type holding the card is not immune.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "BubbleWaterEnergy",
    // The [W] Pokémon this card is attached to recovers from all Special Conditions and can't be
    // affected by any Special Conditions.
    passives: &[
        Passive { origin: RuleSource::Energy, modifier: Modifier::Recover(RecoverSpec { conds: &[], subject: SlotPred::All(&[SlotPred::Holder, SlotPred::TypeIs(ct::WATER)]) }) },
        Passive {
            origin: RuleSource::Energy,
            modifier: Modifier::Prevent(PreventSpec::on(SlotPred::All(&[SlotPred::Holder, SlotPred::TypeIs(ct::WATER)]), EventPred::Kind(EventKind::GainCondition))),
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
