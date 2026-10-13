//! Jumbo Ice Cream (PFL / M2): heal 80 damage from your Active Pokémon that
//! has 3 or more Energy attached.
//!
//! "3 or more Energy attached" counts the Energy provided (user decision 2026-10-12; JP FAQ Jumbo Ice Cream: one Ignition
//! Energy on an Evolution Pokémon, or 2 Basic [G] Energy on Meganium under Wild Growth, is enough; id2237, id2347).
//! Not playable when the Active Pokémon has no damage or less Energy (it would do nothing; D10).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "JumboIce",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[Cond::Cmp(Num::EnergyOn(SlotSel::One(SlotExpr::Active(Who::Me)), EnergyUnit::ProvidedUnits), CmpOp::Ge, Num::Lit(3))],
        steps: &[
            Step::new(heal_active(80)),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
