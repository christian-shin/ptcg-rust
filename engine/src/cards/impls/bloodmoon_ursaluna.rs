//! Bloodmoon Ursaluna (SFA 25): Battle-Hardened — when you play this Pokémon from your hand onto your Bench, you may attach up
//! to 2 Basic [F] Energy from your hand to it. Mad Bite — 100+, 30 more per damage counter on the opponent's Active.
//!
//! Battle-Hardened is an `On(EnterPlay & This(Card) & Source(Hand) & Mode(Rule) & Slot(IsBench))` trigger with an Ability
//! origin. It asks only when a Basic [F] Energy card is in the hand (a "you may" whose yes can't change the game isn't asked);
//! "up to 2" in an Ability is 1 to 2 cards (id2301, RULES.md), then one Attach event per card.
use crate::spec::prelude::*;

const FIGHTING_ENERGY: Pred = Pred::All(&[Pred::BasicEnergy, Pred::Name("Fighting Energy")]);

pub static SPEC: CardSpec = CardSpec {
    class: "BloodmoonUrsaluna",
    // Battle-Hardened: when you play this Pokémon from your hand onto your Bench, you may attach
    // up to 2 Basic [F] Energy from your hand to it.
    triggers: &[Trigger {
        origin: RuleSource::Ability,
        event: Event::On(EventPred::All(&[EventPred::Kind(EventKind::EnterPlay), EventPred::This(Role::Card), EventPred::Source(RulesZone::Hand), EventPred::Mode(EnterMode::Rule), EventPred::Slot(SlotPred::IsBench)])),
        steps: &[Step::new(Op::May(MaySpec {
            asker: Who::Me,
            when: Cond::Nonempty(ZoneRef(Who::Me, Zone::Hand), FIGHTING_ENERGY),
            msg: "WANT_TO_USE_ABILITY",
            yes: &[
                Step::new(Op::Pick(PickSpec {
                    from: ZoneRef(Who::Me, Zone::Hand),
                    predicate: FIGHTING_ENERGY,
                    bounds: Bounds { min: Num::Lit(1), max: Num::Lit(2) },
                    into: 0,
                    msg: "CHOOSE_CARD_TO_ATTACH",
                    ..PickSpec::DEFAULT
                })),
                Step::new(Op::Attach(AttachSpec { from: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::Chosen(0), onto: Some(SlotExpr::This), ..AttachSpec::DEFAULT })),
            ],
            no: &[],
        }))],
    }],
    // Mad Bite: 30 more damage for each damage counter on your opponent's Active Pokémon.
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Mul(&Num::DamageOn(OPP_ACTIVE), &Num::Lit(3)), when: Cond::True }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
