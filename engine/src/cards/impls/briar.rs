//! Briar (SCR): usable only if the opponent has exactly 2 Prize cards left; this turn, if the opponent's Active is
//! Knocked Out by damage from an attack of your Tera Pokémon, take 1 more Prize card.
//!
//! Briar puts an effect on the player (APR C-19): the declared `Modifier::PrizeAdjust` adds the Prize at the
//! KnockOut event (`ko_by` = AttackDamage, the Knocked Out Pokémon is the opponent's Active Pokémon, the Attacking Pokémon is a
//! Tera Pokémon). "During this turn" (`DeclareUntil::EndOfTurn`): the attack's Knock Outs are found by the state check at the end of
//! the attack (`attack_over`), before the turn ends, so the effect is read in time and never reaches a later turn or the opponent's
//! turns.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Briar",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::Printed(&Cond::Cmp(Num::PrizesLeft(Who::Opp), CmpOp::Eq, Num::Lit(2)))],
        steps: &[Step::new(Op::Declare(DeclareSpec { effect: 0, on: DeclareOn::Player(Who::Me), until: DeclareUntil::EndOfTurn }))],
    }),
    // This turn, if the opponent's Active Pokémon is Knocked Out by damage from an attack of your
    // Tera Pokémon, take 1 more Prize card.
    effects: &[Passive {
        origin: RuleSource::TrainerEffect,
        modifier: Modifier::PrizeAdjust(PrizeAdjustSpec {
            delta: 1,
            subject: SlotPred::IsActive,
            by_attack_damage: true,
            by_own_attack: None,
            guard: Cond::AttackerOfKnockOut { who: Who::Opp, pred: Pred::Tag(crate::types::tag::POKEMON_TERA) },
            ..PrizeAdjustSpec::DEFAULT
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
