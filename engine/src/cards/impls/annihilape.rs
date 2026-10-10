//! Annihilape (M5 / PBL): Durable Body — if this Pokémon would be Knocked Out
//! by damage from an attack, flip a coin; heads, it survives with 10 HP.
//! Ghostly Blow — 100, place 5 damage counters on 1 of the opponent's
//! Benched Pokémon.
//!
//! Durable Body is a survive-on-10 replacement in the damage calculation (`damage::survive_on_10`): the coin is flipped
//! when the attack's damage would reach this Pokémon's HP (the damage already on it counts), before the Damage event
//! places it; a copycat whose Ability is blocked doesn't get it. Ghostly Blow is one PlaceCounters event whose cause is
//! the attack: placing counters is an effect of the attack, not damage, so Mist Energy, Empoleon ex, Skeledirge, Hide 'n'
//! Sneak and Battle Cage (on a Benched Pokémon) prevent it, and Weakness and Resistance never apply.
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
