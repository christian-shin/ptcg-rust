//! Mega Heracross ex (PFL 4): Juggernaut Horn - 100+; if this Pokémon was damaged by an attack during your opponent's last
//! turn, this attack does that much more damage. Mountain Ramming - 170; discard the top 2 cards of your opponent's deck.
//!
//! Mountain Ramming is one Discard event of the top 2 cards of the opponent's deck (cause: this attack), after the damage;
//! a deck of 1 card discards that card.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegaHeracrossEx",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::DamageTakenLastTurn(MY_ACTIVE), when: Cond::True }))] },
        AttackSpec { index: 1, steps: &[Step::after_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Opp, Zone::Deck), cards: CardSel::Top(Num::Lit(2)), ..DiscardSpec::DEFAULT }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
