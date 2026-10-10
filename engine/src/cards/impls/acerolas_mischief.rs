//! Acerola's Mischief (MEG): usable only if your opponent has 2 or fewer
//! Prize cards left. Choose 1 of your Pokémon; during your opponent's next
//! turn, prevent all damage from and effects of attacks done to it by your
//! opponent's Pokémon ex.
//!
//! One `Prevent` (`DAMAGE_OR_EFFECTS`) on the marked Pokémon, with the attacker's card as a Pokémon ex of the
//! opponent's as its cause: the attack's damage is zeroed at step 6 (APR C-16) and every event with an effect is
//! refused (Special Conditions, counters, switches, devolving, lasting effects: APR C-04 / C-05 / C-17, id2025, id2155).
//! The marker is a Trainer effect (id2228): it stays when the Pokémon moves to the Bench or switches (the JP FAQ's
//! Ogerpon swap keeps it too), and the opponent's end of turn clears it. Known gap: the JP FAQ ends it when the
//! chosen Pokémon is devolved (Mega Gardevoir ex under Strange Timepiece); the marker stays here.
use crate::spec::prelude::*;

const MISCHIEF: &str = "ACEROLAS_MISCHIEF_MARKER";

pub static SPEC: CardSpec = CardSpec {
    class: "AcerolasMischief",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::Cmp(Num::PrizesLeft(Who::Opp), CmpOp::Le, Num::Lit(2))],
        steps: &[
            Step::new(Op::PickSlot(PickSlotSpec { chooser: Who::Me, among: SlotSel::Pokemon(Who::Me), msg: "CHOOSE_POKEMON" })),
            // A Trainer effect: it stays when the Pokémon moves to the Bench or switches.
            Step::new(Op::SetMarker(SetMarkerSpec { scope: MarkerScope::Slot(SlotExpr::Picked), name: MISCHIEF, source: RuleSource::TrainerEffect })),
        ],
    }),
    passives: &[
        // Prevent all damage from and effects of attacks from your opponent's Pokémon ex done to the chosen Pokémon (every
        // event they cause, the switches included: APR C-04 / C-05, id2025, id2155).
        Passive {
            origin: RuleSource::TrainerEffect,
            modifier: Modifier::Prevent(PreventSpec::on(
                SlotPred::MarkerFromThis(MISCHIEF),
                EventPred::All(&[DAMAGE_OR_EFFECTS, EventPred::Cause(CausePred::All(&[CausePred::By(Who::Opp), CausePred::Kind(crate::cause::CauseKind::Attack), CausePred::Card(Pred::Tag(crate::types::tag::POKEMON_EX_LOWER))]))]),
            )),
        },
    ],
    // The markers end with the opponent's next turn.
    triggers: &[Trigger {
        origin: RuleSource::TrainerEffect,
        event: Event::OnEndTurn(OnEndTurnSpec { whose: Turn::Opp }),
        steps: &[Step::new(Op::ClearMarker(ClearMarkerSpec { scope: MarkerScope::EveryPokemon(Who::Me), name: MISCHIEF, from: MarkerFrom::This }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
