//! Team Rocket's Wobbuffet (DRI): Rocket Mirror — move all damage counters
//! from 1 of your Benched Team Rocket's Pokémon to your opponent's Active
//! Pokémon. Jet Headbutt — 70.
//!
//! Only a damaged Benched Team Rocket's Pokémon can be chosen; with none, the attack does nothing. The move is one
//! MoveCounters event (cause: the attack; both ends are asked, id66): Patrat's Watchful Eye stops it (id2350), a
//! protection on the opponent's Active Pokémon (Mist Energy, Repelling Veil, ...) makes the counters vanish while they
//! still leave the Benched Pokémon (ruling 1665; ids 2257, 79, 393, 1876).
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsWobbuffet",
    attacks: &[AttackSpec {
        index: 0,
        // Rocket Mirror: move all damage counters from 1 of your Benched Team Rocket's Pokémon to your opponent's Active Pokémon.
        steps: &[Step::after_damage(Op::MoveCounters(MoveCountersSpec {
            kind: MoveCountersKind::AllFromOne {
                from: PickSlotSpec {
                    chooser: Who::Me,
                    among: SlotSel::Filtered(&SlotSel::Bench(Who::Me), SlotPred::All(&[SlotPred::Top(Pred::Tag(tag::TEAM_ROCKET)), SlotPred::Damaged])),
                    msg: "CHOOSE_POKEMON_TO_DAMAGE",
                },
                to: SlotTarget::Slot(OPP_ACTIVE),
            },
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
