//! Raticate (POR 61): Scrape Off - 20; before doing damage, discard all Pokémon Tools from your opponent's Active Pokémon.
//! Retaliatory Incisors - 40×; this attack does 40 damage for each damage counter on all of your Benched Rattata.
//!
//! Scrape Off discards the Tools of the Defending Pokémon before the damage (LeavePlay events caused by the attack; Megaton
//! Blower's Tool half, an effect of the attack that Mist Energy stops). Retaliatory Incisors is 4 times the HP of damage on
//! the Benched Pokémon named Rattata (40 per damage counter).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Raticate",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::before_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Opp, Zone::Tools(OPP_ACTIVE)), ..DiscardSpec::DEFAULT }))] },
        AttackSpec {
            index: 1,
            steps: &[Step::before_damage(damage_is(Num::Mul(&Num::DamageSum(SlotSel::Bench(Who::Me), SlotPred::Named("Rattata")), &Num::Lit(4))))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
