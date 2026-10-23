//! Cetitan ex (DRI 65): Snow Camouflage - whenever your opponent plays an Item or Supporter card from their hand, prevent all
//! effects of that card done to this Pokémon. Crushing Press - 140+; you may discard a Stadium in play. If you do, this attack
//! does 140 more damage.
//!
//! Snow Camouflage is a `Prevent` over every event an Item or Supporter of the opponent causes (`CauseKind::Trainer`) on this
//! Pokémon (Fraxure SFA 45's Unnerve, id284; the Trainer's effect on another target still happens: id2154). Crushing Press
//! asks "you may discard a Stadium" when one is in play; on yes the Stadium leaves play for its owner's discard pile (a LeavePlay
//! caused by the attack, before the damage) and "if you do" (`Cond::Done`, APR E-20) adds 140.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "CetitanEx",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(Op::May(MaySpec {
            asker: Who::Me,
            when: Cond::StadiumInPlay(Pred::Any),
            yes: &[Step::new(DISCARD_STADIUM), Step::new(Op::If(IfSpec { cond: Cond::Done, yes: &[Step::new(more_damage_if(140, Cond::True))], no: &[] }))],
            no: &[],
        }))],
    }],
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::Prevent(PreventSpec::on(SlotPred::IsThisPokemon, EventPred::Cause(CausePred::All(&[CausePred::By(Who::Opp), CausePred::Kind(crate::cause::CauseKind::Trainer)])))),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
