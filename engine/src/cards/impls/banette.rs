//! Banette (PBL / M5): Hide 'n' Sneak. Puppet Pull — 80; you may search
//! your deck for a card and put it into your hand, then shuffle.
//!
//! Twinleaf: AFTER_ATTACK opens a ConfirmPrompt (WANT_TO_DRAW_CARDS); on yes
//! with a non-empty deck, SEARCH_DECK_FOR_CARDS_TO_HAND with no filter
//! (min 1, max 1, no cancel; no reveal).
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
            msg: "WANT_TO_DRAW_CARDS",
            yes: &[Step::new(Op::If(IfSpec {
                cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any),
                yes: &[
                    Step::new(Op::Search(SearchSpec {
                        pick: PickSpec { bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
                        destination: SearchDestination::Hand { reveal: false },
                        msg: "",
                        cancel: false,
                        shuffle_first: false,
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
