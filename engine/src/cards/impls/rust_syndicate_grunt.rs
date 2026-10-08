//! Rust Syndicate Grunt (PBL, supporter): only if any of your Pokémon were
//! Knocked Out during your opponent's last turn; discard an Energy from 1 of
//! your opponent's Pokémon.
//!
//! Fixed (phase 4b #38): Twinleaf also required an otherwise empty hand,
//! which the card text does not say; that check is gone. The effect is
//! prevented after the checks, then the card moves to the supporter pile; the
//! Supporter is discarded by CLEAN_UP_SUPPORTER when the prompts finish.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "RustSyndicateGrunt",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::KnockedOutLastTurn { who: Who::Me, tag: None, by_attack: false }, Cond::AnySlot(SlotSel::Pokemon(Who::Opp), SlotPred::HasEnergy)],
        steps: &[Step::new(Op::EnergyChoice(EnergyChoiceSpec {
            from: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Filtered(&SlotSel::Pokemon(Who::Opp), SlotPred::HasEnergy), msg: "CHOOSE_POKEMON_TO_DISCARD_CARDS" }),
            how: EnergyHow::Cards { min: Num::Lit(1), max: Num::Lit(1), kind: EnergyKind::Any, cancel: false, energies_only: true },
            ..EnergyChoiceSpec::DEFAULT
        }))],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
