//! Team Rocket's Zapdos (DRI): Jamming Wave — 30; you may move an Energy
//! from your opponent's Active Pokémon to 1 of their Benched Pokémon.
//! Bad Thunder — 60+, 60 more if this Pokémon has Team Rocket's Energy.
//!
//! Twinleaf: CONFIRMATION_PROMPT; on yes, nothing without an opponent's
//! Benched Pokémon or an Energy card in their Active, else a non-cancellable
//! AttachEnergyPrompt (opponent's Active → TOP_PLAYER Bench, min 1 max 1).
//! Fixed in phase 4b: the callback resolved `getTarget(state, opponent, to)`,
//! which reads TOP_PLAYER as the attacker's Bench (the Energy landed there,
//! even in an empty slot); it now uses the attacker as the perspective, so the
//! Energy goes to the opponent's Benched Pokémon. Bad Thunder checks the
//! Energy name "Team Rocket's Energy" (it compared "Team Rocket Energy", so
//! the +60 never applied).
//!
//! Fixed (phase 4b, R7F-7; ruling 1843): the move was a plain MOVE_CARDS, so
//! Mist Energy (or any effect that prevents the effects of attacks) on the
//! Defending Pokémon did not stop it; it is now a MoveOpponentEnergyEffect
//! (target = the Defending Pokémon), like Elgyem's Slight Shift.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsZapdos",
    attacks: &[
        AttackSpec {
            index: 0,
            // You may move an Energy from your opponent's Active Pokémon to 1 of their Benched Pokémon.
            steps: &[Step::after_damage(Op::May(MaySpec {
                asker: Who::Me,
                when: Cond::All(&[Cond::AnySlot(SlotSel::Bench(Who::Opp), SlotPred::Any), Cond::Slot(OPP_ACTIVE, SlotPred::HasEnergy)]),
                msg: "WANT_TO_USE_ABILITY",
                yes: &[Step::new(Op::EnergyChoice(EnergyChoiceSpec {
                    from: SlotTarget::Slot(OPP_ACTIVE),
                    how: EnergyHow::ToBench { min: Num::Lit(1), max: Num::Lit(1), same_target: false, via_effect: true },
                    to: EnergyDest::Stay,
                    ..EnergyChoiceSpec::DEFAULT
                }))],
                no: &[],
            }))],
        },
        AttackSpec { index: 1, steps: &[Step::before_damage(more_damage_if(60, Cond::Slot(MY_ACTIVE, SlotPred::HasEnergyNamed("Team Rocket's Energy"))))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
