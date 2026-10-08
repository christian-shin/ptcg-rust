//! Slaking ex (SSP): Born to Slack — if your opponent has no Pokémon ex or
//! Pokémon V in play, this Pokémon can't attack. Great Swing — 280; discard
//! an Energy from this Pokémon.
//!
//! Twinleaf: any AttackEffect while this card is the attacker's Active
//! Pokémon throws BLOCKED_BY_ABILITY unless the opponent has an ex / V /
//! VMAX / VSTAR / V-UNION in play or the ability is blocked. Great Swing
//! prices the discard as [C] with a non-cancellable ChooseEnergyPrompt and
//! reduces a DiscardCardsEffect on `player.active`.
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "Slakingex",
    // Born to Slack: if your opponent has no Pokémon ex or Pokémon V in play, this Pokémon can't attack.
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::BlockAttack(BlockAttackSpec {
            on: AttackBlockOn::ActiveAttack,
            unless: Cond::InPlay(Who::Opp, PlayScope::All, Pred::OneOf(&[Pred::Tag(tag::POKEMON_EX_LOWER), Pred::Tag(tag::POKEMON_V), Pred::Tag(tag::POKEMON_VMAX), Pred::Tag(tag::POKEMON_VSTAR), Pred::Tag(tag::POKEMON_VUNION)])),
            error: "BLOCKED_BY_ABILITY",
        }),
    }],
    attacks: &[AttackSpec {
        index: 0,
        // Great Swing: discard an Energy from this Pokémon (priced as [C]).
        steps: &[Step::after_damage(Op::EnergyChoice(EnergyChoiceSpec { how: EnergyHow::Cost { n: Num::Lit(1), ty: ct::COLORLESS }, ..EnergyChoiceSpec::DEFAULT }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
