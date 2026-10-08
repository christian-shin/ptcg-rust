//! Scramble Switch (PLS, ACE SPEC): switch your Active Pokémon with 1 of your
//! Benched Pokémon. Then, you may move as many Energy attached to the old
//! Active Pokémon to the new Active Pokémon as you like.
//!
//! Twinleaf: throws without a Benched Pokémon; the card moves to the
//! Supporter area (not the discard pile) and the play is prevented; a
//! ChoosePokemonPrompt over the Bench (fixed in phase 4b, R3: it can't be
//! cancelled; cancelling used to burn the card with no switch). When the
//! Active has Energy, a ChooseCardsPrompt over the Active's cards (Energy,
//! 0..all, no cancel) moves the chosen Energy to the chosen Benched Pokémon;
//! then the silent `switchPokemon(target)` runs.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "ScrambleSwitch",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[Cond::Cmp(Num::BenchCount(Who::Me), CmpOp::Gt, Num::Lit(0))],
        steps: &[
            Step::new(Op::PickSlot(PickSlotSpec { chooser: Who::Me, among: SlotSel::Bench(Who::Me), msg: "CHOOSE_POKEMON_TO_SWITCH" })),
            // You may move as many Energy as you like from the old Active Pokémon to the new one.
            Step::new(Op::EnergyChoice(EnergyChoiceSpec {
                how: EnergyHow::Cards { min: Num::Lit(0), max: Num::CardsOn(MY_ACTIVE), kind: EnergyKind::Any, cancel: false, energies_only: false },
                to: EnergyDest::Slot(SlotExpr::Chosen),
                ..EnergyChoiceSpec::DEFAULT
            })),
            Step::new(Op::Switch(SwitchSpec { side: Who::Me, chooser: Who::Me, kind: SwitchKind::ChosenSilent, msg: "", required: false })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
