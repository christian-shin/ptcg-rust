//! Dialga (MEG 95): Beam — 30. Chrono Burst — 80+; you may shuffle all
//! Energy attached to this Pokémon into your deck and have this attack do 80
//! more damage.
//!
//! Fixed (ruling 1822): Twinleaf asked nothing when the Active had no Energy, so
//! the 80 more damage was unreachable; the ConfirmPrompt is now always asked and
//! yes adds 80 (with no Energy nothing moves and the deck is not shuffled).
//! Twinleaf: a ConfirmPrompt, then one MOVE_CARDS per mapped card (captured
//! before the prompt; Active to deck), SHUFFLE_DECK and `effect.damage += 80`
//! (after the shuffle prompt is opened, before it resolves).
//! R7A (rulings 1580, 1846): the Energy goes back into the deck, and the deck is shuffled, after the damage (`move_cards_after_damage`, `shuffle_deck_after_damage`).
use crate::spec::prelude::*;

const CHRONO: &str = "DIALGA_CHRONO_BURST_MARKER";
const ATTACHED: ZoneRef = ZoneRef(Who::Me, Zone::Attached(MY_ACTIVE));

pub static SPEC: CardSpec = CardSpec {
    class: "Dialga",
    // Chrono Burst: you may shuffle all Energy attached to this Pokémon into your deck and have
    // this attack do 80 more damage (the Energy goes back into the deck after the damage).
    attacks: &[AttackSpec {
        index: 1,
        steps: &[
            Step::before_damage(Op::May(MaySpec {
                asker: Who::Me,
                when: Cond::True,
                msg: "WANT_TO_USE_ABILITY",
                yes: &[
                    Step::new(Op::SetMarker(SetMarkerSpec { scope: MarkerScope::Player(Who::Me), name: CHRONO, source: RuleSource::CardRule })),
                    Step::new(more_damage_if(80, Cond::True)),
                ],
                no: &[],
            })),
            Step::after_damage(Op::If(IfSpec {
                cond: Cond::HasMarker { who: Who::Me, name: CHRONO, from: MarkerFrom::This },
                yes: &[
                    Step::new(Op::ClearMarker(ClearMarkerSpec { scope: MarkerScope::Player(Who::Me), name: CHRONO, from: MarkerFrom::This })),
                    Step::new(Op::Snapshot(SnapshotSpec { zone: ATTACHED, predicate: Pred::Energy, into: 0 })),
                    Step::new(Op::Move(MoveSpec { from: ATTACHED, to: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::Chosen(0), ..MoveSpec::DEFAULT })),
                    Step::new(Op::If(IfSpec {
                        cond: Cond::Chosen(0),
                        yes: &[Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true }))],
                        no: &[],
                    })),
                ],
                no: &[],
            })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
