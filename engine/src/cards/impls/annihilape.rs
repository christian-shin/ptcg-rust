//! Annihilape (M5 / PBL): Durable Body — if this Pokémon would be Knocked Out
//! by damage from an attack, flip a coin; heads, it survives with 10 HP.
//! Ghostly Blow — 100, place 5 damage counters on 1 of the opponent's
//! Benched Pokémon.
//!
//! Fixed (phase 4b, X1-1): the coin's callback used to set
//! `surviveOnTenHPReason` after the flip's wait prompt, i.e. after the
//! PutDamageEffect was already applied, so the flip happened but never saved
//! the Pokémon (and the would-KO test ignored the damage already on it). The
//! flip is now read right away (SURVIVE_ON_TEN_ON_COIN_FLIP). Also fixed: when the code runs
//! for a copycat (a copied Ghostly Blow's session), `IS_ABILITY_BLOCKED` is
//! true for the copycat, so Durable Body no longer applies to it (it used to
//! throw here for a copycat whose card has no powers).
//!
//! Fixed (phase 4b, R2): Ghostly Blow placed its counters with a
//! PlaceDamageCountersEffect (an Ability effect), which Mist Energy, Empoleon
//! ex and Skeledirge do not prevent; it now uses a PutCountersEffect (an
//! effect of the attack) on the chosen Benched Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Annihilape@PBL",
    // Durable Body: if this Pokémon would be Knocked Out by damage from an attack, flip a coin;
    // heads, it survives with 10 HP.
    passives: &[Passive { origin: RuleSource::Ability, modifier: Modifier::SurviveOnTen(SurviveOnTenSpec { kind: SurviveKind::OnCoin }) }],
    // Ghostly Blow: place 5 damage counters on 1 of the opponent's Benched Pokémon.
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::PlaceCounters(PlaceCountersSpec {
            target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Bench(Who::Opp), msg: "CHOOSE_POKEMON_TO_DAMAGE" }),
            counters: Num::Lit(5)
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
