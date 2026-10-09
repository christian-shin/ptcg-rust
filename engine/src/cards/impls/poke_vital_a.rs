//! Poké Vital A (SFA, ACE SPEC): heal 150 damage from 1 of your Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "PokeVitalA",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[Step::new(Op::Heal(HealSpec {
            target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Filtered(&SlotSel::Pokemon(Who::Me), SlotPred::Damaged), msg: "CHOOSE_POKEMON_TO_HEAL" }),
            hp: Num::Lit(150),
            clear_conditions: false,
        }))],
    }),
    passives: &[Passive { origin: RuleSource::CardRule, modifier: Modifier::Prevent(PreventSpec { what: PreventWhat::ThisCardFromDiscard, ..PreventSpec::NONE }) }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
