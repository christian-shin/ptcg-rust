//! Genesect (M2 / PFL 8): Bug's Cannon — choose 1 of your opponent's
//! Pokémon; 20 damage to it for each [G] Energy attached to this Pokémon.
//! Speed Attack — 110.
//!
//! Twinleaf: counts GRASS / ANY `provides` on `player.active`, then a
//! non-cancellable ChoosePokemonPrompt (min 1, max 1) and
//! DAMAGE_OPPONENT_POKEMON (DealDamage on the Active, PutDamage on the
//! Bench), even for 0 damage. Two `Genesect` classes exist; this port is
//! bound to PFL.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Genesect@PFL",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(Op::DamageSlot(DamageSlotSpec { target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Pokemon(Who::Opp), msg: "CHOOSE_POKEMON_TO_DAMAGE" }), hp: Num::Mul(&Num::EnergyOn(SlotSel::One(SlotExpr::Active(Who::Me)), EnergyUnit::Provided(ct::GRASS)), &Num::Lit(20)), target_damage_mul: 0, calc: DamageCalc::Auto, when: Cond::True })),
        ] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
