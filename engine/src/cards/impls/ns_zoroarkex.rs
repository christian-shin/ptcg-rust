//! N's Zoroark ex (JTG 98): Trade: you must discard a card from your hand in order to use this Ability. Once during your
//! turn, you may draw 2 cards. Night Joker: choose 1 of your Benched N's Pokémon's attacks and use it as this attack.
//!
//! Trade: the discard is a cost (`Step::cost`, APR E-30; id2115: it can't be used when no card can be drawn), the draw is the
//! effect. Night Joker copies a Benched N's Pokémon's attack, another N's Zoroark ex's Night Joker included (a copy attack
//! can copy a copy attack: compendium id656, JP FAQ Slowking SCR 58); that copied Night Joker chooses again among the
//! attacks not already being used in this attack (id657; `copy_attack.rs`).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "NsZoroarkEx",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(Op::CopyAttack(CopyAttackSpec {
            from: Who::Me,
            predicate: Pred::Tag(crate::types::tag::NS),
            scope: CopyScope::Bench,
        }))],
    }],
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn,
        // Discarding is a cost, drawing the effect (id2115).
        needs: &[],
        steps: &[
            Step::new(Op::Pick(PickSpec { from: ZoneRef(Who::Me, Zone::Hand), bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, into: 0, ..PickSpec::DEFAULT })),
            Step::cost(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT })),
            Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(2)) })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
