//! Slowking (SCR): Seek Inspiration — discard the top card of your deck; if
//! it is a Pokémon without a Rule Box, choose 1 of its attacks and use it as
//! this attack (COPY_ATTACK_FROM_POKEMON_LIST, see `copy_attack.rs`).
//! Super Psy Bolt — 120.
//!
//! Fixed (phase 4b, R3): the attack choice can't be cancelled (it used to
//! allow it, so the copied attack could be skipped), and nothing is chosen
//! when every attack of the Pokémon is locked.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Slowking",
    attacks: &[AttackSpec {
        index: 0,
        // Discard the top card of your deck; if it is a Pokémon without a Rule Box, choose 1 of its attacks and use it as this attack.
        steps: &[Step::before_damage(Op::If(IfSpec {
            cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any),
            yes: &[
                Step::new(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::Top(Num::Lit(1)), into: Some(0), ..DiscardSpec::DEFAULT })),
                Step::new(Op::If(IfSpec {
                    cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Scratch(0)), Pred::All(&[Pred::Pokemon, Pred::Not(&Pred::RuleBox)])),
                    yes: &[Step::new(Op::CopyAttack(CopyAttackSpec { from: Who::Me, predicate: Pred::Any, retries: 1, scope: CopyScope::Register(0) }))],
                    no: &[],
                })),
            ],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
