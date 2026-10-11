//! Chi-Yu (TWM): Allure — draw 2 cards. Ground Melter — 60+; if a Stadium is
//! in play, 60 more damage, then discard that Stadium.
//!
//! Allure is a Draw event by the attack (as many as the deck has, APR H). Ground Melter's Stadium leaves play to its
//! owner's discard pile (a LeavePlay, user decision D1).
//!
//! Fixed (phase 4b, R2): the Stadium was discarded in the attack handler,
//! before the damage step (Full Metal Lab's reduction was lost); the text says
//! "Then, discard that Stadium", so the discard now runs in AfterAttackEffect.
use crate::spec::prelude::*;

const MY_STADIUM: ZoneRef = ZoneRef(Who::Me, Zone::Stadium);
const OPP_STADIUM: ZoneRef = ZoneRef(Who::Opp, Zone::Stadium);

const STADIUM_IN_PLAY: Cond = Cond::Any(&[Cond::Nonempty(MY_STADIUM, Pred::Any), Cond::Nonempty(OPP_STADIUM, Pred::Any)]);

pub static SPEC: CardSpec = CardSpec {
    class: "ChiYu@TWM",
    attacks: &[
        // Allure: draw 2 cards.
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(2)) }))],
        },
        // Ground Melter: if a Stadium is in play, 60 more damage, then discard that Stadium.
        AttackSpec {
            index: 1,
            steps: &[
                Step::before_damage(more_damage_if(60, STADIUM_IN_PLAY)),
                Step::after_damage(Op::If(IfSpec {
                    cond: Cond::Nonempty(MY_STADIUM, Pred::Any),
                    yes: &[Step::new(Op::Discard(DiscardSpec { from: MY_STADIUM, cards: CardSel::All, ..DiscardSpec::DEFAULT }))],
                    no: &[],
                })),
                Step::after_damage(Op::If(IfSpec {
                    cond: Cond::Nonempty(OPP_STADIUM, Pred::Any),
                    yes: &[Step::new(Op::Discard(DiscardSpec { from: OPP_STADIUM, cards: CardSel::All, ..DiscardSpec::DEFAULT }))],
                    no: &[],
                })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
