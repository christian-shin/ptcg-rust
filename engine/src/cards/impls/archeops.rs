//! Archeops (WHT 51): Ancient Wing - once during your turn, if this Pokémon is in the Active Spot, you may devolve 1 of your
//! opponent's evolved Pokémon by putting the highest Stage Evolution card on it into your opponent's hand. Rock Throw - 100.
//!
//! An Ability (once per turn) that needs this Pokémon in the Active Spot. One evolved Pokémon of the opponent's is chosen
//! (`PickSlot`), then one Devolve event by the Ability puts its top card into the opponent's hand (APR C-13; a Pokémon that
//! prevents the effects of Abilities stays evolved; the devolved Pokémon keeps its counters and is Knocked Out at the state check
//! if they reach its HP; Espeon ex PRE 34's Amazez, Strange Timepiece).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Archeops",
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn,
        needs: &[Cond::IsActive(SlotExpr::This)],
        steps: &[
            Step::new(Op::PickSlot(PickSlotOp { pick: PickSlotSpec { chooser: Who::Me, among: SlotSel::Filtered(&SlotSel::Pokemon(Who::Opp), SlotPred::Evolved) }, purpose: PromptPurpose::Devolve })),
            Step::new(Op::Devolve(DevolveSpec { slot: SlotExpr::Picked, destination: ZoneRef(Who::Opp, Zone::Hand), chooser: None })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
