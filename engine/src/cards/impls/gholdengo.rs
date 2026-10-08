//! Gholdengo (SSP 131): Strike It Rich — 30+; 90 more if this Pokémon
//! evolved from Gimmighoul during this turn. Surf Back — 100; you may shuffle
//! this Pokémon and all attached cards into your deck.
//!
//! Strike It Rich checks that the card under the user is Gimmighoul and that it
//! evolved this turn (E-24; Twinleaf used to check only `pokemonPlayedTurn ===
//! state.turn`, which Zoroark's Foul Play copy satisfied by evolving from Zorua). Surf Back:
//! ConfirmPrompt, then MOVE_CARDS of the whole Active to the deck,
//! `player.active.clearEffects()` and a ShuffleDeckPrompt (no trailing wait).
//! Fixed (W1-A): Surf Back used to run in the attack handler, i.e. before the
//! damage step, so the Pokémon was already in the deck while its 100 damage
//! was dealt (no Weakness/Resistance, lost tools, a crash in handlers that
//! read the attacker); it now runs on the AfterAttackEffect, like Tuck Tail.
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
                Step::after_damage(Op::May(MaySpec { asker: Who::Me, when: Cond::True, msg: "WANT_TO_USE_ABILITY", yes: &[Step::new(Op::RemoveFromPlay(RemoveFromPlaySpec { slot: MY_ACTIVE, destination: ZoneRef(Who::Me, Zone::Deck) })), Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck) }))], no: &[] })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
