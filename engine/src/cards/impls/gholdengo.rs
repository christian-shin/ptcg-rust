//! Gholdengo (SSP 131): Strike It Rich — 30+; if this Pokémon evolved from
//! Gimmighoul during this turn, 90 more damage. Surf Back — 100; you may shuffle
//! this Pokémon and all attached cards into your deck.
//!
//! Rule: Strike It Rich checks that the card under this Pokémon is Gimmighoul and
//! that it was played this turn (E-24; a copy of the attack, Zoroark's Foul Play,
//! checks the copier). Surf Back runs after the attack's damage: the optional
//! RemoveFromPlay (one LeavePlay to the deck, cause: this attack) and the shuffle.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Gholdengo",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Lit(90), when: Cond::Slot(SlotExpr::This, SlotPred::All(&[SlotPred::PlayedThisTurn, SlotPred::CardBelowThis("Gimmighoul")])) })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(Op::May(MaySpec { asker: Who::Me, when: Cond::True, msg: "WANT_TO_USE_ABILITY", yes: &[Step::new(Op::RemoveFromPlay(RemoveFromPlaySpec { slot: MY_ACTIVE, destination: ZoneRef(Who::Me, Zone::Deck) })), Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true }))], no: &[] })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
