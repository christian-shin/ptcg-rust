//! Turtonator (POR 17): Shell Spikes — if this Pokémon is in the Active Spot and is damaged by an attack from your opponent's Pokémon
//! (even if this Pokémon is Knocked Out), discard an Energy from the Attacking Pokémon. Heat Breath — 80+; flip a coin. If heads,
//! this attack does 80 more damage.
//!
//! Shell Spikes is a trigger over the Damage event of an opponent's attack on this Pokémon while it is the Active Pokémon
//! (`DamagedActive`, id1992), resolved after the attack's own effects (APR E-03): you choose 1 Energy card of the Attacking Pokémon,
//! wherever it is now, and it is discarded by the Ability (a LeavePlay caused by the Ability), so Mist Energy doesn't stop it
//! (id2310: it prevents effects of attacks only). Heat Breath flips before the damage and adds 80 on heads.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TurtonatorPOR",
    attacks: &[AttackSpec { index: 0, steps: &[Step::before_damage(Op::Coin(CoinSpec { heads: &[Step::new(more_damage_if(80, Cond::True))], ..CoinSpec::DEFAULT }))] }],
    triggers: &[Trigger {
        origin: RuleSource::Ability,
        event: Event::On(EventPred::All(&[DAMAGE_BY_OPP_ATTACKS, EventPred::Slot(SlotPred::Holder), EventPred::DamagedActive])),
        steps: &[Step::new(Op::DiscardEnergy(DiscardEnergySpec {
            target: SlotTarget::Slot(SlotExpr::CausePokemon),
            selection: EnergySelection::Cards { min: Num::Lit(1), max: Num::Lit(1), kind: EnergyKind::Any, cancel: false, energies_only: true },
            ..DiscardEnergySpec::DEFAULT
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
