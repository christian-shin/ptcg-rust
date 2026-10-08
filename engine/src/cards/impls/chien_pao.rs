//! Chien-Pao (SSP): Snow Sink - when you play this Pokémon from your hand
//! onto your Bench, you may discard a Stadium in play. Icicle Loop - 120;
//! put an Energy attached to this Pokémon into your hand.
//!
//! Twinleaf quirks kept: the ability check runs while the card is still in
//! hand and the prompt is offered on any turn. Fixed (phase 4b, X1-2): Icicle
//! Loop used to run in the attack handler, i.e. before the damage step, and
//! asked for energy covering [C][C] (up to 2 Energy went to the hand, and a
//! lone 2-unit Special Energy could not be chosen); it now runs on the
//! AfterAttackEffect and asks for exactly 1 attached Energy card.
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
        event: Event::OnEnterPlay(OnEnterPlaySpec { method: EnterMethod::Play }),
        steps: &[Step::new(Op::May(MaySpec {
            asker: Who::Me,
            when: Cond::Any(&[Cond::Nonempty(MY_STADIUM, Pred::Any), Cond::Nonempty(OPP_STADIUM, Pred::Any)]),
            msg: "WANT_TO_USE_ABILITY",
            yes: &[
                Step::new(Op::If(IfSpec {
                    cond: Cond::Nonempty(MY_STADIUM, Pred::Any),
                    yes: &[Step::new(Op::Move(MoveSpec { from: MY_STADIUM, to: ZoneRef(Who::Me, Zone::Discard), cards: CardSel::All, ..MoveSpec::DEFAULT }))],
                    no: &[],
                })),
                Step::new(Op::If(IfSpec {
                    cond: Cond::Nonempty(OPP_STADIUM, Pred::Any),
                    yes: &[Step::new(Op::Move(MoveSpec { from: OPP_STADIUM, to: ZoneRef(Who::Opp, Zone::Discard), cards: CardSel::All, ..MoveSpec::DEFAULT }))],
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
                msg: "CHOOSE_CARD_TO_HAND",
                ..PickSpec::DEFAULT
            })),
            Step::after_damage(Op::Move(MoveSpec { from: ATTACHED, to: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::Chosen(0), ..MoveSpec::DEFAULT })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
