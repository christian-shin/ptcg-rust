//! Slaking ex (SSP 147): Born to Slack - if your opponent has no Pokémon ex or Pokémon V in play, this Pokémon can't attack.
//! Great Swing - 280; discard an Energy from this Pokémon.
//!
//! Born to Slack is a lock over this Pokémon's own UseAttack (`Kind(UseAttack) & This(Card)`), held while the opponent has no ex /
//! V / VMAX / VSTAR / V-UNION in play (`LockWhile::Unless`, an Ability lock stops it). Great Swing's discard is an effect after
//! the damage: a LeavePlay of 1 Energy (priced as [C]) caused by this attack.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "SlakingEx",
    // Born to Slack: if your opponent has no Pokémon ex or Pokémon V in play, this Pokémon can't attack.
    // A lock over its own UseAttack while the opponent has no Pokémon ex or V in play.
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::BlockUse(BlockUseSpec {
            binds: Binds::Owner,
            lock: LockDecl::on(EventPred::All(&[EventPred::Kind(EventKind::UseAttack), EventPred::This(Role::Card)]), "BLOCKED_BY_ABILITY"),
            while_: &[LockWhile::Unless(&Cond::InPlay(
                Who::Opp,
                PlayScope::All,
                Pred::OneOf(&[Pred::PokemonEx, Pred::PokemonV]),
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
