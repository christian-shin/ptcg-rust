//! Mega Heracross ex (PFL): Juggernaut Horn — 100+; more damage equal to the
//! damage this Pokémon took during your opponent's last turn. Mountain
//! Ramming — 170; discard the top 2 cards of your opponent's deck.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegaHeracrossex",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::DamageTakenLastTurn(MY_ACTIVE), when: Cond::True }))] },
        AttackSpec { index: 1, steps: &[Step::after_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Opp, Zone::Deck), cards: CardSel::Top(Num::Lit(2)), ..DiscardSpec::DEFAULT }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
