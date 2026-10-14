//! Cynthia's Garchomp ex (ASC 111): Corkscrew Dive - 100; you may draw cards until you have 6 cards in your hand.
//! Draconic Buster - 260; discard all Energy from this Pokémon.
//!
//! The draw is one Draw event (cause: this attack), asked only when it would draw something (6 or more cards in hand, or an
//! empty deck, ask nothing). Draconic Buster discards every Energy card on the Pokémon after the damage: a LeavePlay per
//! card (D1).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "CynthiasGarchompex",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::May(MaySpec {
                asker: Who::Me,
                // Nothing to ask with 6 or more cards in hand or an empty deck.
                when: Cond::All(&[
                    Cond::Cmp(Num::ZoneSize(ZoneRef(Who::Me, Zone::Hand)), CmpOp::Lt, Num::Lit(6)),
                    Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any),
                ]),
                purpose: PromptPurpose::MayDrawUntilHandSize,
                yes: &[Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::UntilHandSize(Num::Lit(6)) }))],
                no: &[],
            }))],
        },
        AttackSpec {
            index: 1,
            steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(SlotExpr::Active(Who::Me)), selection: EnergySelection::AllProvided, ..DiscardEnergySpec::DEFAULT }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
