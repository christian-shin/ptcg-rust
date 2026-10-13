//! Delibird (MEG 105): Quick Gift (if you go first, you can use it during your first turn): search your deck for a card and
//! put it into your hand. Then, shuffle your deck. Gentle Slap: 30.
//!
//! "If you go first, you can use this attack during your first turn" is `AttackFlags { first_turn: Attack(0) }` on Quick Gift,
//! read by the UseAttack's first-turn check.
//!
//! One card of any kind goes into the hand (a PutIntoHand event, this attack its cause; nothing is revealed); an empty deck
//! can't be searched (id840: the attack then does nothing, no shuffle); otherwise the deck is shuffled (APR E-19).
use crate::spec::prelude::*;

const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);

pub static SPEC: CardSpec = CardSpec {
    class: "DelibirdMEGPool",
    passives: &[Passive { origin: RuleSource::CardRule, modifier: Modifier::AttackFlags(AttackFlagsSpec { first_turn: Some(FirstTurnAttack::Attack(0)), shred: None }) }],
    // Quick Gift: search your deck for a card and put it into your hand, then shuffle.
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::If(IfSpec {
            cond: Cond::Nonempty(DECK, Pred::Any),
            yes: &[
                Step::new(Op::Search(SearchSpec {
                    pick: PickSpec { bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
                    destination: SearchDestination::Hand { reveal: false },
                    msg: "",
                    cancel: false,
                })),
                Step::new(Op::Shuffle(ShuffleSpec { zone: DECK, wait: true })),
            ],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
