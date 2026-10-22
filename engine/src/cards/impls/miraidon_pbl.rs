//! Miraidon (PBL 28): Photon Cord - if this Pokémon is in the Active Spot and is Knocked Out by damage from an attack from your
//! opponent's Pokémon, move up to 2 Basic [L] Energy cards from this Pokémon to 1 of your Benched Pokémon. Thunder - 90; this
//! Pokémon also does 30 damage to itself.
//!
//! Photon Cord is a trigger on this card's own KnockOut (Maractus JTG) while it was in the Active Spot when the damage was done
//! (id1992), moving 1 or 2 of its Basic [L] Energy cards to one Benched Pokémon (MoveEnergy events caused by the Ability, Heavy Baton's
//! move): "up to" takes at least 1 when there is one (id2399: only an attack may choose 0), onto a different Pokémon (id2400);
//! nothing without a Benched Pokémon. Thunder's self damage is the usual Damage event on this Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MiraidonPBL",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(self_damage(30))] }],
    triggers: &[Trigger {
        origin: RuleSource::Ability,
        event: Event::On(EventPred::All(&[
            EventPred::Kind(EventKind::KnockOut),
            EventPred::This(Role::Card),
            EventPred::DamagedActive,
            EventPred::KoBy(KoBy::AttackDamage),
            EventPred::Cause(CausePred::All(&[CausePred::By(Who::Opp), CausePred::Kind(crate::cause::CauseKind::Attack)])),
        ])),
        steps: &[Step::new(Op::DiscardEnergy(DiscardEnergySpec {
            target: SlotTarget::Slot(SlotExpr::This),
            selection: EnergySelection::ToBench { min: Num::Lit(1), max: Num::Lit(2), same_target: true, via_effect: false, kind: EnergyKind::BasicProvides(ct::LIGHTNING) },
            to: EnergyDest::Stay,
            ..DiscardEnergySpec::DEFAULT
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
