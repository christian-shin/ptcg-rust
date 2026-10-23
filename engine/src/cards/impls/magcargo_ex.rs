//! Magcargo ex (TWM 29): Tera - as long as this Pokémon is on your Bench, prevent all damage done to this Pokémon by
//! attacks (both yours and your opponent's). Hot Magma - 70; your opponent's Active Pokémon is now Burned. Ground Burn -
//! 140+; discard the top card of each player's deck. This attack does 140 more damage for each Energy card discarded in
//! this way.
//!
//! The Tera rule is `TERA_RULE` (Dragapult ex TWM). Hot Magma burns the Defending Pokémon after the damage. Ground Burn
//! discards the top card of each deck before the damage (two Discard events caused by the attack, kept in registers 0 and
//! 1) and adds 140 per Energy card (any Energy card, Special too) among them (Mega Abomasnow ex MEG 36).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MagcargoEx",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::after_damage(inflict(&[SpecialCondition::Burned]))] },
        AttackSpec {
            index: 1,
            steps: &[
                Step::before_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::Top(Num::Lit(1)), into: Some(0) })),
                Step::before_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Opp, Zone::Deck), cards: CardSel::Top(Num::Lit(1)), into: Some(1) })),
                Step::before_damage(Op::Damage(DamageSpec {
                    op: DamageOp::Add,
                    hp: Num::Mul(&Num::Add(&Num::CardCount(ZoneRef(Who::Me, Zone::Scratch(0)), Pred::Energy), &Num::CardCount(ZoneRef(Who::Me, Zone::Scratch(1)), Pred::Energy)), &Num::Lit(140)),
                    when: Cond::True,
                })),
            ],
        },
    ],
    passives: &[Passive { origin: RuleSource::CardRule, modifier: Modifier::Prevent(TERA_RULE) }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
