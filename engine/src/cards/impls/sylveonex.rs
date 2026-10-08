//! Sylveon ex (SSP, Tera): Magical Charm - 160; during your opponent's next
//! turn, the Defending Pokémon's attacks do 100 less damage. Angelite - choose
//! 2 of your opponent's Benched Pokémon; they shuffle those Pokémon and all
//! attached cards into their deck; can't be used if 1 of your Pokémon used
//! Angelite during your last turn. Tera: no attack damage while on the Bench.
//!
//! Twinleaf: Angelite does nothing with an empty opposing Bench; with the
//! marker it throws BLOCKED_BY_EFFECT; the marker is added before the prompt
//! (phase 4b: no longer again in its callback, where a copy by Clefable's
//! Metronome added a marker named `undefined` after the delegation scope
//! ended); each chosen Pokémon is moved to the opponent's deck and followed by
//! its own ShuffleDeckPrompt for the opponent (no trailing wait). The marker
//! pair (ANGELITE / CLEAR_ANGELITE) is kept on the player and cleared at the
//! end of the following turn of that player.
//! Phase 4b (R6): "If 1 of your Pokémon used Angelite during your last turn"
//! is checked by marker name only (any Sylveon ex or copy of the attack; it
//! used to be keyed to the card, so a second copy could still use it); the
//! marker is set and checked before the empty-Bench return (using the attack
//! counts even when it does nothing); "choose 2" is exactly 2 (min 1 for a
//! single Benched Pokémon only).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Sylveonex",
    passives: &[Passive { origin: RuleSource::CardRule, modifier: Modifier::PreventDamage(PreventDamageSpec { how: PreventHow::Tera, ..PreventDamageSpec::DEFAULT }) }],
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::DealsLessDamage(100) }))] },
        AttackSpec {
            index: 1,
            steps: &[
                // Can't be used if 1 of your Pokémon used Angelite during your last turn.
                Step::before_damage(Op::Fail(FailSpec { when: Cond::HasMarker { who: Who::Me, name: "ANGELITE_MARKER", from: MarkerFrom::Any }, error: "BLOCKED_BY_EFFECT" })),
                Step::before_damage(Op::SetMarker(SetMarkerSpec { scope: MarkerScope::Player(Who::Me), name: "ANGELITE_MARKER", source: RuleSource::CardRule })),
                // Choose 2 of your opponent's Benched Pokémon; they shuffle those Pokémon and all attached cards into their deck.
                Step::after_damage(Op::EachSlot(EachSlotSpec {
                    among: SlotSel::Bench(Who::Opp),
                    choose: Some(ChooseN { chooser: Who::Me, min: Num::Min(&Num::Lit(2), &Num::BenchCount(Who::Opp)), max: Num::Lit(2), msg: "CHOOSE_POKEMON_TO_DAMAGE" }),
                    what: EachWhat::ShuffleIntoDeck,
                    ..EachSlotSpec::DEFAULT
                })),
            ],
        },
    ],
    triggers: &[Trigger {
        origin: RuleSource::CardRule,
        event: Event::OnEndTurn(OnEndTurnSpec { whose: Turn::Owner }),
        // The marker is kept through the following turn of its player, then cleared.
        steps: &[
            Step::new(Op::If(IfSpec {
                cond: Cond::HasMarker { who: Who::Me, name: "CLEAR_ANGELITE_MARKER", from: MarkerFrom::This },
                yes: &[
                    Step::new(Op::ClearMarker(ClearMarkerSpec { scope: MarkerScope::Player(Who::Me), name: "ANGELITE_MARKER", from: MarkerFrom::Any })),
                    Step::new(Op::ClearMarker(ClearMarkerSpec { scope: MarkerScope::Player(Who::Me), name: "CLEAR_ANGELITE_MARKER", from: MarkerFrom::This })),
                ],
                no: &[],
            })),
            Step::new(Op::If(IfSpec {
                cond: Cond::HasMarker { who: Who::Me, name: "ANGELITE_MARKER", from: MarkerFrom::Any },
                yes: &[Step::new(Op::SetMarker(SetMarkerSpec { scope: MarkerScope::Player(Who::Me), name: "CLEAR_ANGELITE_MARKER", source: RuleSource::CardRule }))],
                no: &[],
            })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
