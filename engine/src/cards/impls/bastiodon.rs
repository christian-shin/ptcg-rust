//! Bastiodon (PBL 62): Ancient Bulwark - as long as this Pokémon is on your Bench, prevent all damage done to each of your Pokémon by
//! attacks from your opponent's Pokémon that have 2 or less Energy attached. Hammer In - 160.
//!
//! An Ability `Prevent` over the Damage events of the opponent's attacks done to your Pokémon (`SlotPred::OnMySide`), whose
//! attacking Pokémon has 2 or fewer Energy attached (`SlotPred::EnergyAtMost`, the Energy it provides, read on the attacker where
//! it is when the damage is done), guarded by this Pokémon being on the Bench (`PreventSpec::while_`). An Ability lock turns it off.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Bastiodon",
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::Prevent(
            PreventSpec::on(
                SlotPred::OnMySide,
                EventPred::All(&[DAMAGE_BY_OPP_ATTACKS, EventPred::Cause(CausePred::Pokemon(SlotPred::EnergyAtMost(2)))]),
            )
            .while_(Cond::Slot(SlotExpr::This, SlotPred::IsBench)),
        ),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
