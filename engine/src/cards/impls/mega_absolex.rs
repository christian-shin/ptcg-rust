//! Mega Absol ex (M1L / MEG 86): Terminal Period — if the opponent's Active
//! has exactly 6 damage counters, it is Knocked Out. Claw of Darkness — 200,
//! the opponent reveals their hand and you discard a card from it.
//!
//! Terminal Period is a KnockOut event on the opponent's Active Pokémon when it has exactly 60 damage (the Knock Out
//! waits for the state check, D1); Mist Energy prevents it (id2427). Claw of Darkness discards the chosen card from the
//! opponent's revealed hand after the damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegaAbsolex",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::before_damage(Op::KnockOut(KnockOutSpec {
                target: OPP_ACTIVE,
                when: Cond::Cmp(Num::DamageOn(OPP_ACTIVE), CmpOp::Eq, Num::Lit(60)),
            }))],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(Op::Pick(PickSpec {
                    chooser: Who::Me,
                    from: ZoneRef(Who::Opp, Zone::Hand),
                    bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
                    into: 0,
                    msg: "CHOOSE_CARD_TO_DISCARD",
                    ..PickSpec::DEFAULT
                })),
                Step::after_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Opp, Zone::Hand), cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
