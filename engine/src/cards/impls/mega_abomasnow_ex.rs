//! Mega Abomasnow ex (MEG 36): Hammer-lanche — 100×; discard the top 6 cards of your deck, and this attack does 100 damage for each
//! Basic [W] Energy card that you discarded in this way. Frost Barrier — 200; during your opponent's next turn, this Pokémon takes
//! 30 less damage from attacks (after applying Weakness and Resistance).
//!
//! Hammer-lanche discards the top 6 cards (a Discard event caused by the attack, before the damage) and sets the damage to 100
//! times the Basic [W] Energy cards among them (a Special Energy card isn't a Basic Energy card). Frost Barrier arms the
//! lasting "takes less damage" effect.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegaAbomasnowEx",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::Top(Num::Lit(6)), into: Some(0) })),
                Step::before_damage(damage_is(Num::Mul(&Num::CardCount(ZoneRef(Who::Me, Zone::Scratch(0)), Pred::All(&[Pred::BasicEnergy, Pred::Provides(ct::WATER)])), &Num::Lit(100)))),
            ],
        },
        AttackSpec { index: 1, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::TakesLessDamage(30) }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
