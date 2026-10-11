//! Slowking (SCR 58): Seek Inspiration - discard the top card of your deck, and if that card is a Pokémon that doesn't
//! have a Rule Box, choose 1 of its attacks and use it as this attack. Super Psy Bolt - 120.
//!
//! One Discard event from the deck (cause: this attack); the discarded card is remembered. A copied attack
//! is this attack using that attack (see `copy_attack.rs`): the choice can't be cancelled, and nothing is chosen when
//! every attack of the Pokémon is locked.
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
