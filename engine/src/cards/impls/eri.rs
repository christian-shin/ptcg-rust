//! Eri (TEF): your opponent reveals their hand. Discard up to 2 Item cards
//! you find there.
//!
//! Twinleaf: the card moves itself to the supporter pile, then to the discard
//! when the prompt is answered; chosen Items are discarded one MOVE_CARDS each.
//! Fixed (phase 4b, R3): min 1 when the opponent's hand holds an Item (it was 0).
use crate::spec::prelude::*;

const OPP_HAND: ZoneRef = ZoneRef(Who::Opp, Zone::Hand);

pub static SPEC: CardSpec = CardSpec {
    class: "Eri",
    // Your opponent reveals their hand. Discard up to 2 Item cards you find there (at least 1 when
    // there is one, unless used as the effect of an attack, ruling 1844).
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::Nonempty(OPP_HAND, Pred::Any)],
        steps: &[
            Step::new(Op::Pick(PickSpec {
                from: OPP_HAND,
                predicate: Pred::Item,
                bounds: Bounds {
                    min: Num::If(&Cond::All(&[Cond::Nonempty(OPP_HAND, Pred::Item), Cond::Not(&Cond::TrainerViaAttack)]), &Num::Lit(1), &Num::Lit(0)),
                    max: Num::Lit(2),
                },
                into: 0,
                soft: true,
                msg: "CHOOSE_CARD_TO_DISCARD",
                ..PickSpec::DEFAULT
            })),
            Step::new(Op::Discard(DiscardSpec { from: OPP_HAND, cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
