//! Maractus (JTG): Explosive Needle — if this Pokémon is in the Active Spot
//! and is Knocked Out by damage from an attack from your opponent's Pokémon,
//! put 6 damage counters on the Attacking Pokémon. Corner — 20; the
//! Defending Pokémon can't retreat during your opponent's next turn.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Maractus@JTG",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::PreventRetreat }))] }],
    triggers: &[Trigger {
        origin: RuleSource::Ability,
        event: Event::OnKnockOut(OnKnockOutSpec { which: KoWhich::ThisByAttack }),
        // 6 damage counters on the Attacking Pokémon, placed by the Ability (not damage): effects of
        // the opponent's Abilities on the target are prevented by Hide 'n' Sneak and the like.
        steps: &[Step::new(Op::PlaceCounters(PlaceCountersSpec { target: SlotTarget::Slot(SlotExpr::Picked), counters: Num::Lit(6) }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
