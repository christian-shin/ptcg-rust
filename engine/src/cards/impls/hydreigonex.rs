//! Hydreigon ex (SSP, Tera): Crashing Headbutt — 200; discard the top 3
//! cards of your opponent's deck. Obsidian — 130; also 130 damage to 2 of
//! your opponent's Benched Pokémon.
//!
//! Twinleaf: THIS_ATTACK_DOES_X_DAMAGE_TO_X_OF_YOUR_OPPONENTS_POKEMON with
//! min = max = min(2, benched). `AttackEffect.target` is never set, so the
//! `effect.target === effect.opponent.active` branch never fires and every
//! target gets a PutDamageEffect.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Hydreigonex",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(Op::Move(MoveSpec { from: ZoneRef(Who::Opp, Zone::Deck), to: ZoneRef(Who::Opp, Zone::Discard), cards: CardSel::Top(Num::Lit(3)), ..MoveSpec::DEFAULT })),
        ] },
        AttackSpec { index: 1, steps: &[
            Step::after_damage(Op::EachSlot(EachSlotSpec { among: SlotSel::Bench(Who::Opp), choose: Some(ChooseN { chooser: Who::Me, min: Num::Min(&Num::Lit(2), &Num::SlotCount(SlotSel::Bench(Who::Opp), SlotPred::Any)), max: Num::Min(&Num::Lit(2), &Num::SlotCount(SlotSel::Bench(Who::Opp), SlotPred::Any)), msg: "CHOOSE_POKEMON_TO_DAMAGE" }), what: EachWhat::Damage(DamageCalc::Put), amount: Num::Lit(130), ..EachSlotSpec::DEFAULT })),
        ] },
    ],
    passives: &[
        Passive { origin: RuleSource::CardRule, modifier: Modifier::PreventDamage(PreventDamageSpec { how: PreventHow::Tera, ..PreventDamageSpec::DEFAULT }) }
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
