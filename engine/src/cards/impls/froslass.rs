//! Froslass (TWM): Freezing Shroud — during Pokémon Checkup, put 1 damage
//! counter on each Pokémon in play that has any Abilities (excluding any
//! Froslass). Frost Smash — 60.
//!
//! Each Froslass in play whose Ability works puts its counters, separately (id2302: two Froslass put 2 counters, each
//! applying its Ability), in its owner's part of the Checkup, its owner's Pokémon first. The counters are a
//! PlaceCounters by Froslass's Ability: Hide 'n' Sneak keeps them off the opponent's Poltchageist (official JP FAQ,
//! Poltchageist / Froslass: いいえ、できません), Battle Cage off the opponent's Benched Pokémon but not the owner's
//! own (official JP FAQ, Battle Cage / Froslass: 自分のベンチ…はい、のせます).
use crate::spec::prelude::*;

const NOT_FROSLASS_WITH_ABILITY: SlotPred = SlotPred::All(&[SlotPred::Not(&SlotPred::Named("Froslass")), SlotPred::HasAbility]);

pub static SPEC: CardSpec = CardSpec {
    class: "Froslass",
    triggers: &[Trigger {
        origin: RuleSource::Ability,
        event: Event::OnCheckup(OnCheckupSpec {}),
        steps: &[
            Step::new(Op::ForEach(ForEachSpec {
                over: SlotSel::Filtered(&SlotSel::Pokemon(Who::Me), NOT_FROSLASS_WITH_ABILITY),
                body: &[Step::new(Op::PlaceCounters(PlaceCountersSpec { target: SlotTarget::Slot(SlotExpr::Picked), counters: Num::Lit(1) }))],
            })),
            Step::new(Op::ForEach(ForEachSpec {
                over: SlotSel::Filtered(&SlotSel::Pokemon(Who::Opp), NOT_FROSLASS_WITH_ABILITY),
                body: &[Step::new(Op::PlaceCounters(PlaceCountersSpec { target: SlotTarget::Slot(SlotExpr::Picked), counters: Num::Lit(1) }))],
            })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
