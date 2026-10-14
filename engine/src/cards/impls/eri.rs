//! Eri (TEF 146): your opponent reveals their hand. Discard up to 2 Item cards you find there.
//!
//! The Items are chosen from the opponent's hand (at least 1 when there is one, unless the Supporter's effect is used by
//! an attack: "up to" may take 0, id2376) and discarded from it (a Discard
//! event over the opponent's hand, this Supporter or the attack using it its cause). The hand is revealed first (a Reveal
//! routine, D2); with no cards in it Eri can't be played (id1492).
use crate::spec::prelude::*;

const OPP_HAND: ZoneRef = ZoneRef(Who::Opp, Zone::Hand);

pub static SPEC: CardSpec = CardSpec {
    class: "Eri",
    // Your opponent reveals their hand. Discard up to 2 Item cards you find there (at least 1 when
    // there is one, unless used as the effect of an attack, id2376).
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[
            // "Your opponent reveals their hand"
            Step::new(Op::Reveal(RevealSpec { cards: RevealWhat::Zone(ZoneRef(Who::Opp, Zone::Hand)), by: Who::Opp, to: Who::Me, when_empty: true })),
            Step::new(Op::Pick(PickSpec {
                from: OPP_HAND,
                predicate: Pred::Item,
                bounds: Bounds {
                    min: Num::If(&Cond::All(&[Cond::Nonempty(OPP_HAND, Pred::Item), Cond::Not(&Cond::TrainerViaAttack)]), &Num::Lit(1), &Num::Lit(0)),
                    max: Num::Lit(2),
                },
                into: 0,
                soft: true,
                ..PickSpec::DEFAULT
            })),
            Step::new(Op::Discard(DiscardSpec { from: OPP_HAND, cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
