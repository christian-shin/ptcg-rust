//! Cynthia's Garchomp ex (DRI, ASC): Corkscrew Dive - 100; you may draw cards
//! until you have 6 cards in your hand. Draconic Buster - 260; discard all
//! Energy from this Pokémon.
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
                msg: "WANT_TO_DRAW_UNTIL_6",
                yes: &[Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::UntilHandSize(Num::Lit(6)) }))],
                no: &[],
            }))],
        },
        AttackSpec {
            index: 1,
            steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec {
                target: SlotExpr::Active(Who::Me),
                selection: EnergySelection::AllProvided,
            }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
