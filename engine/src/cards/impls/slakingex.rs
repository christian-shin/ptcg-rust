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
    // A lock over its own UseAttack while the opponent has no Pokémon ex or V in play (events batch 7).
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::BlockUse(BlockUseSpec {
            binds: Binds::Owner,
            lock: LockDecl::on(EventPred::All(&[EventPred::Kind(EventKind::UseAttack), EventPred::This(Role::Card)]), "BLOCKED_BY_ABILITY"),
            while_: &[LockWhile::Unless(&Cond::InPlay(
                Who::Opp,
                PlayScope::All,
                Pred::OneOf(&[Pred::Tag(tag::POKEMON_EX_LOWER), Pred::Tag(tag::POKEMON_V), Pred::Tag(tag::POKEMON_VMAX), Pred::Tag(tag::POKEMON_VSTAR), Pred::Tag(tag::POKEMON_VUNION)]),
            ))],
            ability: true,
        }),
    }],
    attacks: &[AttackSpec {
        index: 0,
        // Great Swing: discard an Energy from this Pokémon (priced as [C]).
        steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { selection: EnergySelection::Cost { n: Num::Lit(1), ty: ct::COLORLESS }, ..DiscardEnergySpec::DEFAULT }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
