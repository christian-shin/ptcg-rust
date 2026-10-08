//! Team Rocket's Wobbuffet (DRI): Rocket Mirror — move all damage counters
//! from 1 of your Benched Team Rocket's Pokémon to your opponent's Active
//! Pokémon. Jet Headbutt — 70.
//!
//! Twinleaf: every Bench position that is not a damaged Team Rocket's Pokémon
//! (empty positions included) is blocked; with no damaged one the attack does
//! nothing.
//!
//! Fixed (phase 4b, R7F-5; ruling 1665): the callback moved `damage` directly,
//! so Mist Energy, Repelling Veil, ... on the opponent's Active did not stop
//! it. Like Cofagrigus WHT it now checks MoveDamageCountersEffect and
//! reduces a MoveCountersAttackEffect (source = the Benched Pokémon, target =
//! the opponent's Active): the counters always leave the Benched Pokémon, and
//! are placed only when the move was not prevented.
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsWobbuffet",
    attacks: &[AttackSpec {
        index: 0,
        // Rocket Mirror: move all damage counters from 1 of your Benched Team Rocket's Pokémon to your opponent's Active Pokémon.
        steps: &[Step::after_damage(Op::MoveCounters(MoveCountersSpec {
            kind: MoveCountersKind::AllToSlot {
                from: PickSlotSpec {
                    chooser: Who::Me,
                    among: SlotSel::Filtered(&SlotSel::Bench(Who::Me), SlotPred::All(&[SlotPred::Top(Pred::Tag(tag::TEAM_ROCKET)), SlotPred::Damaged])),
                    msg: "CHOOSE_POKEMON_TO_DAMAGE",
                },
                to: OPP_ACTIVE,
            },
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
