//! Gourgeist ex (CRI 41): Horrifying Rondo - 30+; this attack does 50 more damage for each of your Benched Pokémon that has any damage
//! counters on it. Ghostly Touch - 140; discard a random card from your opponent's hand.
//!
//! Horrifying Rondo counts the Benched Pokémon of yours with damage counters before the damage. Ghostly Touch discards 1 random card
//! of the opponent's hand after the damage (a Discard caused by the attack; nothing with an empty hand).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "GourgeistEx",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Mul(&Num::SlotCount(SlotSel::Bench(Who::Me), SlotPred::Damaged), &Num::Lit(50)), when: Cond::True }))],
        },
        AttackSpec {
            index: 1,
            steps: &[Step::after_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Opp, Zone::Hand), cards: CardSel::Random(Num::Lit(1)), ..DiscardSpec::DEFAULT }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
