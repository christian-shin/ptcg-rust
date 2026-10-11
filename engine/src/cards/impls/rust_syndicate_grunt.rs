//! Rust Syndicate Grunt (PBL 81, Supporter): "You can use this card only if any of your Pokémon were Knocked Out during your
//! opponent's last turn." Discard an Energy from 1 of your opponent's Pokémon.
//!
//! The Knock Out is the printed condition (`Cond::Printed`); an opposing Pokémon with an Energy is what usability adds. The
//! discard is a LeavePlay (to the discard pile) of the Energy you choose, caused by this Supporter (D1).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "RustSyndicateGrunt",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::Printed(&Cond::KnockedOutLastTurn { who: Who::Me, by_attack_damage: false, tag: None }), Cond::AnySlot(SlotSel::Pokemon(Who::Opp), SlotPred::HasEnergy)],
        steps: &[Step::new(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Filtered(&SlotSel::Pokemon(Who::Opp), SlotPred::HasEnergy), msg: "CHOOSE_POKEMON_TO_DISCARD_CARDS" }), selection: EnergySelection::Cards { min: Num::Lit(1), max: Num::Lit(1), kind: EnergyKind::Any, cancel: false, energies_only: true }, ..DiscardEnergySpec::DEFAULT }))],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
