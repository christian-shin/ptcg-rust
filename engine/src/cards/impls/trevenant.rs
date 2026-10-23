//! Trevenant (CRI 39): Cursed Roots - 30; during your opponent's next turn, Energy can't be attached from your opponent's
//! hand to the Defending Pokémon. Overwhelming Pain - 60+; this attack does 10 more damage for each damage counter on all
//! of your opponent's Pokémon.
//!
//! Cursed Roots arms a lasting lock on the Defending Pokémon over the Attach of an Energy card from the hand, by the rule
//! or by an effect (Walrein's Frigid Fangs and Budew's Item lock are the same lasting form; it ends when the Pokémon moves
//! to the Bench, APR C-15). Overwhelming Pain adds 10 per damage counter on the opponent's Pokémon: the HP of damage on all
//! of them.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Trevenant",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::Arm(ArmSpec {
                what: Lasting::Lock(LastingLockSpec::on_defending(&LockDecl::on(
                    EventPred::All(&[EventPred::Kind(EventKind::Attach), EventPred::Source(RulesZone::Hand), EventPred::Card(Pred::Energy)]),
                    "BLOCKED_BY_EFFECT",
                ))),
            }))],
        },
        AttackSpec {
            index: 1,
            steps: &[Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::DamageSum(SlotSel::Pokemon(Who::Opp), SlotPred::Any), when: Cond::True }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
