//! Mega Absol ex (MEG 86): Terminal Period: if your opponent's Active Pokémon has exactly 6 damage counters on it, that Pokémon
//! is Knocked Out. Claw of Darkness: 200; your opponent reveals their hand, and you discard a card you find there.
//!
//! Terminal Period is a KnockOut event on the opponent's Active Pokémon when it has exactly 60 damage (the Knock Out waits for
//! the state check, D1); Mist Energy prevents it (id2427). Claw of Darkness: after the damage the attacker chooses a card of the
//! opponent's hand and it is discarded from it (a Discard event over the opponent's hand, this attack its cause). The hand is
//! revealed first (a Reveal routine, D2).
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
                // "Your opponent reveals their hand, and you discard a card you find there."
                Step::after_damage(Op::Reveal(RevealSpec { cards: RevealWhat::Zone(ZoneRef(Who::Opp, Zone::Hand)), by: Who::Opp, to: Who::Me, when_empty: true })),
                Step::after_damage(Op::Pick(PickSpec {
                    chooser: Who::Me,
                    from: ZoneRef(Who::Opp, Zone::Hand),
                    bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
                    into: 0,
                    purpose: Some(PromptPurpose::DiscardCard),
                    ..PickSpec::DEFAULT
                })),
                Step::after_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Opp, Zone::Hand), cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
