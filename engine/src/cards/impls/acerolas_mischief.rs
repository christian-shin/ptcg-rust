//! Acerola's Mischief (MEG): usable only if your opponent has 2 or fewer
//! Prize cards left. Choose 1 of your Pokémon; during your opponent's next
//! turn, prevent all damage from and effects of attacks done to it by your
//! opponent's Pokémon ex.
//!
//! Twinleaf: every AbstractAttackEffect whose target carries this card's
//! marker is prevented when its player does not own the target and the
//! source's top Pokémon is an ex. The markers are cleared at the end of the
//! turn of the player holding the clear marker (the opponent).
//!
//! Fixed (phase 4b, R7C, ruling 1730): the marker on the Pokémon is a Trainer effect
//! (`SourceType::Trainer`), so it stays when the Pokémon moves to the Bench, switches,
//! evolves or devolves (`remove_all_except_trainer_effects`); it used to be wiped with
//! the attack effects.
use crate::spec::prelude::*;

const MISCHIEF: &str = "ACEROLAS_MISCHIEF_MARKER";

pub static SPEC: CardSpec = CardSpec {
    class: "AcerolasMischief",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::Cmp(Num::PrizesLeft(Who::Opp), CmpOp::Le, Num::Lit(2))],
        steps: &[
            Step::new(Op::PickSlot(PickSlotSpec { chooser: Who::Me, among: SlotSel::Pokemon(Who::Me), msg: "CHOOSE_POKEMON" })),
            // A Trainer effect: it stays when the Pokémon moves, switches, evolves or devolves.
            Step::new(Op::SetMarker(SetMarkerSpec { scope: MarkerScope::Slot(SlotExpr::Picked), name: MISCHIEF, source: RuleSource::TrainerEffect })),
        ],
    }),
    passives: &[Passive {
        origin: RuleSource::TrainerEffect,
        modifier: Modifier::PreventAttackEffects(PreventAttackEffectsSpec {
            subject: SlotPred::MarkerFromThis(MISCHIEF),
            attacker: SlotPred::Tag(crate::types::tag::POKEMON_EX_LOWER),
            needs_source_pokemon: false,
            damage_too: true,
            ..PreventAttackEffectsSpec::DEFAULT
        }),
    }],
    // The markers end with the opponent's next turn.
    triggers: &[Trigger {
        origin: RuleSource::TrainerEffect,
        event: Event::OnEndTurn(OnEndTurnSpec { whose: Turn::Opp }),
        steps: &[Step::new(Op::ClearMarker(ClearMarkerSpec { scope: MarkerScope::EveryPokemon(Who::Me), name: MISCHIEF, from: MarkerFrom::This }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
