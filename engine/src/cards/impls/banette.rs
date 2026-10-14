//! Banette (PBL 34): Hide 'n' Sneak. Puppet Pull: 80; you may search your deck for a card and put it into your hand. Then,
//! shuffle your deck.
//!
//! Hide 'n' Sneak is `Modifier::Prevent(HIDE_N_SNEAK)` (see shuppet.rs). Puppet Pull asks first (a "may"); on yes with a
//! non-empty deck (id840) one card of any kind goes into the hand (a PutIntoHand event, this attack its cause; nothing is
//! revealed, the text doesn't say so), then the deck is shuffled (APR E-19).
use super::shuppet::HIDE_N_SNEAK;
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Banette",
    // Hide 'n' Sneak.
    passives: &[
        Passive { origin: RuleSource::Ability, modifier: Modifier::Prevent(HIDE_N_SNEAK) },
    ],
    // Puppet Pull: you may search your deck for a card and put it into your hand, then shuffle.
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::May(MaySpec {
            asker: Who::Me,
            when: Cond::True,
            purpose: PromptPurpose::DrawCount,
            yes: &[Step::new(Op::If(IfSpec {
                cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any),
                yes: &[
                    Step::new(Op::Search(SearchSpec {
                        pick: PickSpec { bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
                        destination: SearchDestination::Hand { reveal: false },
                        cancel: false,
                    })),
                    Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
                ],
                no: &[],
            }))],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
