//! Gwynn (M5 / PBL): discard up to 2 Pokémon that don't have a Rule Box from
//! your hand, and draw 3 cards for each card discarded.
//!
//! Twinleaf: needs 1 selectable card (else CANNOT_PLAY_THIS_CARD, after the
//! card moved to the supporter zone; fixed in phase 4b, R4: it needed 2 though
//! the text says up to 2); the prompt is `min: 1, max: 2`.
//!
//! R7C: as the effect of an attack (Mr. Mime's Look-Alike Show) the prompt is min 0
//! (rulings 1844, 1853); played from the hand it is min 1.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Gwynn",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[
            Step::new(Op::Pick(PickSpec { from: ZoneRef(Who::Me, Zone::Hand), predicate: Pred::All(&[Pred::Pokemon, Pred::Not(&Pred::RuleBox)]), bounds: Bounds { min: Num::If(&Cond::ViaAttack, &Num::Lit(0), &Num::Lit(1)), max: Num::Lit(2) }, into: 0, msg: "CHOOSE_CARD_TO_DISCARD", ..PickSpec::DEFAULT })),
            Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Hand), to: ZoneRef(Who::Me, Zone::Discard), cards: CardSel::Chosen(0), ..MoveSpec::DEFAULT })),
            Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Mul(&Num::RegCount(0), &Num::Lit(3))) })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
