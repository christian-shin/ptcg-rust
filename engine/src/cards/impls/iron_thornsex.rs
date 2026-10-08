//! Iron Thorns ex (TWM / PRE): Initialization — as long as this Pokémon is in
//! the Active Spot, Pokémon with a Rule Box in play (both yours and your
//! opponent's) have no Abilities, except for Future Pokémon. Volt Cyclone —
//! 140; move an Energy from this Pokémon to 1 of your Benched Pokémon.
//!
//! Twinleaf HANDLE_ABILITY_LOCK ('remove' mode, allowUseFromHand,
//! allowUseFromDiscard, respectExemptFromInitialize, error
//! BLOCKED_BY_ABILITY): strips Abilities from CheckPokemonPowersEffect and
//! throws on PowerEffect when this card is the top card of either Active, the
//! checked card sits on a Pokémon slot or is still in the hand (a Pokémon
//! being benched or evolved: it is in play, so its on-play Ability is locked
//! too; fixed in phase 4b), is not Future and has a Rule Box,
//! and Initialization itself applies (a real PowerEffect for it, by the
//! player whose Active it is, must not throw; on a PowerEffect also an
//! EffectOfAbilityEffect against the target slot must keep its target).
//! Initialization's own EffectOfAbilityEffect drops the target when it is a
//! Future Pokémon. Volt Cyclone prompts after the attack (AfterAttackEffect)
//! for 1 Energy to move to the Bench.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "IronThornsex",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(Op::If(IfSpec { cond: Cond::AnySlot(SlotSel::Bench(Who::Me), SlotPred::Any), yes: &[Step::new(Op::Attach(AttachSpec { from: ZoneRef(Who::Me, Zone::Active), predicate: Pred::Energy, bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, ..AttachSpec::DEFAULT }))], no: &[] })),
        ] },
    ],
    passives: &[
        Passive { origin: RuleSource::Ability, modifier: Modifier::ActiveLock(ActiveLock::Initialization) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
