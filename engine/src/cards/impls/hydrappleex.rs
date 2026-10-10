//! Hydrapple ex (SCR): Ripening Charge — once during your turn, attach a
//! Basic [G] Energy card from your hand to 1 of your Pokémon and heal 30
//! damage from it. Syrup Storm — 30+, 30 more for each [G] Energy attached
//! to all of your Pokémon.
//!
//! Fixed (R1-18): cancelling the attach prompt doesn't use the Ability up (the
//! once-per-turn marker and the ABILITY_USED board effect used to be set
//! anyway). Twinleaf quirk kept: only the first transfer is processed. Syrup Storm counts `provides` entries equal to
//! [G] or ANY across every CheckProvidedEnergy map.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Hydrappleex",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Mul(&Num::EnergyOn(SlotSel::Pokemon(Who::Me), EnergyUnit::Provided(ct::GRASS)), &Num::Lit(30)), when: Cond::True })),
        ] },
    ],
    powers: &[PowerSpec {
        index: 0,
        once: Once::No,
        needs: &[Cond::Not(&Cond::HasMarker { who: Who::Me, name: "RIPE_CHARGE_MARKER", from: MarkerFrom::This }), Cond::Nonempty(ZoneRef(Who::Me, Zone::Hand), Pred::All(&[Pred::BasicEnergy, Pred::Provides(ct::GRASS)]))],
        steps: &[
            Step::new(Op::Attach(AttachSpec { from: ZoneRef(Who::Me, Zone::Hand), predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Grass Energy")]), slots: AttachSlots::ActiveBench, bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, cancel: true, ..AttachSpec::DEFAULT })),
            Step::new(Op::If(IfSpec { cond: Cond::Cmp(Num::Last, CmpOp::Gt, Num::Lit(0)), yes: &[Step::new(Op::AbilityUsed(AbilityUsedSpec { marker: Some("RIPE_CHARGE_MARKER") })), Step::new(Op::Heal(HealSpec { target: SlotTarget::Slot(SlotExpr::Attached), hp: Num::Lit(30), clear_conditions: false }))], no: &[] })),
        ],
    }],
    triggers: &[
        Trigger { origin: RuleSource::Ability, event: Event::OnEndTurn(OnEndTurnSpec { whose: Turn::Owner }), steps: &[Step::new(Op::ClearMarker(ClearMarkerSpec { scope: MarkerScope::Player(Who::Me), name: "RIPE_CHARGE_MARKER", from: MarkerFrom::This }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
