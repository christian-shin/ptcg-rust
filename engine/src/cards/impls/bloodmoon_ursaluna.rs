//! Bloodmoon Ursaluna (SFA 25): Battle-Hardened — when you play this Pokémon
//! from your hand onto your Bench, you may attach up to 2 Basic [F] Energy
//! from your hand to it. Mad Bite — 100+, 30 more per damage counter on the
//! opponent's Active.
//!
//! Fixed (phase 4b): with no Basic [F] Energy in hand the Ability does nothing
//! (no prompt); it used to ask, then throw CANNOT_USE_POWER in the callback.
//! Otherwise a ConfirmPrompt, then a cancel-free ChooseCardsPrompt (1-2: phase 4b
//! R7E, "up to 2" in an Ability takes at least 1, rulings 1853/1778; it was 0-2).
use crate::spec::prelude::*;

const FIGHTING_ENERGY: Pred = Pred::All(&[Pred::BasicEnergy, Pred::Name("Fighting Energy")]);

pub static SPEC: CardSpec = CardSpec {
    class: "BloodmoonUrsaluna",
    // Battle-Hardened: when you play this Pokémon from your hand onto your Bench, you may attach
    // up to 2 Basic [F] Energy from your hand to it.
    triggers: &[Trigger {
        origin: RuleSource::Ability,
        event: Event::OnEnterPlay(OnEnterPlaySpec { method: EnterMethod::Play }),
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
                Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::Chosen(0), place: Place::AttachTo(SlotExpr::This), ..MoveSpec::DEFAULT })),
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
