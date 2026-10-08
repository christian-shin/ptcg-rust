//! N's Zoroark ex (JTG): Trade - discard a card from your hand to draw 2,
//! once during your turn; Night Joker - choose 1 of your Benched N's
//! Pokémon's attacks and use it as this attack.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "NsZoroarkex",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(Op::CopyAttack(CopyAttackSpec {
            from: Who::Me,
            predicate: Pred::All(&[Pred::Tag(crate::types::tag::NS), Pred::Not(&Pred::Name("N's Zoroark ex"))]),
            retries: 1,
            scope: CopyScope::Bench,
        }))],
    }],
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn("TRADE_MARKER"),
        // Discarding is a cost, drawing the effect (ruling 1640).
        needs: &[Cond::Nonempty(ZoneRef(Who::Me, Zone::Hand), Pred::Any), Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any)],
        steps: &[
            Step::new(Op::Pick(PickSpec { from: ZoneRef(Who::Me, Zone::Hand), bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, into: 0, msg: "CHOOSE_CARD_TO_DISCARD", ..PickSpec::DEFAULT })),
            Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Hand), to: ZoneRef(Who::Me, Zone::Discard), cards: CardSel::Chosen(0), ..MoveSpec::DEFAULT })),
            Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(2)) })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
