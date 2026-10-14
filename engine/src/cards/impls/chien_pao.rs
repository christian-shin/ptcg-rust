//! Chien-Pao (SSP 56): Snow Sink — when you play this Pokémon from your hand onto your Bench during your turn, you may discard
//! a Stadium in play. Icicle Loop — 120; put an Energy attached to this Pokémon into your hand.
//!
//! Snow Sink is an `On(EnterPlay & This(Card) & Source(Hand) & Mode(Rule) & Slot(IsBench))` trigger with an Ability origin
//! (playing it from the hand is the player's own action during their turn); the Stadium leaves play by a Discard event caused
//! by the Ability. Icicle Loop runs after the damage and puts exactly 1 attached Energy card into the hand (a PutIntoHand event
//! caused by the attack).
use crate::spec::prelude::*;

const MY_STADIUM: ZoneRef = ZoneRef(Who::Me, Zone::Stadium);
const OPP_STADIUM: ZoneRef = ZoneRef(Who::Opp, Zone::Stadium);
const ATTACHED: ZoneRef = ZoneRef(Who::Me, Zone::Attached(MY_ACTIVE));

pub static SPEC: CardSpec = CardSpec {
    class: "ChienPao",
    // Snow Sink: when you play this Pokémon from your hand onto your Bench, you may discard a
    // Stadium in play.
    triggers: &[Trigger {
        origin: RuleSource::Ability,
        event: Event::On(EventPred::All(&[EventPred::Kind(EventKind::EnterPlay), EventPred::This(Role::Card), EventPred::Source(RulesZone::Hand), EventPred::Mode(EnterMode::Rule), EventPred::Slot(SlotPred::IsBench)])),
        steps: &[Step::new(Op::May(MaySpec {
            asker: Who::Me,
            when: Cond::Any(&[Cond::Nonempty(MY_STADIUM, Pred::Any), Cond::Nonempty(OPP_STADIUM, Pred::Any)]),
            purpose: PromptPurpose::Activate,
            yes: &[
                Step::new(Op::If(IfSpec {
                    cond: Cond::Nonempty(MY_STADIUM, Pred::Any),
                    yes: &[Step::new(Op::Discard(DiscardSpec { from: MY_STADIUM, cards: CardSel::All, ..DiscardSpec::DEFAULT }))],
                    no: &[],
                })),
                Step::new(Op::If(IfSpec {
                    cond: Cond::Nonempty(OPP_STADIUM, Pred::Any),
                    yes: &[Step::new(Op::Discard(DiscardSpec { from: OPP_STADIUM, cards: CardSel::All, ..DiscardSpec::DEFAULT }))],
                    no: &[],
                })),
            ],
            no: &[],
        }))],
    }],
    // Icicle Loop: put an Energy attached to this Pokémon into your hand.
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::after_damage(Op::Pick(PickSpec {
                from: ATTACHED,
                predicate: Pred::Energy,
                bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
                into: 0,
                purpose: Some(PromptPurpose::PutIntoHand),
                ..PickSpec::DEFAULT
            })),
            Step::after_damage(Op::PutIntoHand(PutIntoHandSpec { from: ATTACHED, cards: CardSel::Chosen(0), ..PutIntoHandSpec::DEFAULT })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
