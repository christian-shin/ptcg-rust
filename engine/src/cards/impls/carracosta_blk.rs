//! Carracosta (BLK 23): Mighty Shell - prevent all damage from and effects of attacks done to this Pokémon by your opponent's
//! Pokémon that have any Special Energy attached. Big Bite - 150; during your opponent's next turn, the Defending Pokémon can't
//! retreat.
//!
//! Mighty Shell is one `Prevent` over `DAMAGE_OR_EFFECTS` on this Pokémon, restricted to the cause: an attack of the opponent's
//! Pokémon that has a Special Energy attached now (`SlotPred::HasSpecialEnergy`, read on the attacker where it is when the event
//! happens; Rabsca's Spherical Shield, Cornerstone Mask Ogerpon ex's Stance). Big Bite arms the lasting `CANT_RETREAT` lock on the
//! Defending Pokémon (Bruxish SSP 49).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "CarracostaBLK",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_defending(&CANT_RETREAT)) }))] }],
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::Prevent(PreventSpec::on(
            SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon]),
            EventPred::All(&[DAMAGE_OR_EFFECTS, EventPred::Cause(CausePred::All(&[CausePred::By(Who::Opp), CausePred::Kind(crate::cause::CauseKind::Attack), CausePred::Pokemon(SlotPred::HasSpecialEnergy)]))]),
        )),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
