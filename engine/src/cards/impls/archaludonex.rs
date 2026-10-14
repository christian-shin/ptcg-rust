//! Archaludon ex (SSP 130): Assemble Alloy — when you play this Pokémon from your hand to evolve 1 of your Pokémon, you may
//! attach up to 2 Basic [M] Energy from your discard pile to your [M] Pokémon in any way you like. Metal Defender — 220; during
//! your opponent's next turn this Pokémon has no Weakness.
//!
//! Assemble Alloy is an `On(Evolve & This(Card) & Source(Hand))` trigger with an Ability origin (Rare Candy included; an
//! Ability lock turns it off). Its "up to 2" is 1 to 2 cards in an Ability (id2301, RULES.md), each
//! attached by its own Attach event with the Ability as its cause, to a Pokémon that is [M] as the game checks its type
//! (`TargetScan::EffectiveTypes`: a Pokémon whose type an effect changed counts by the changed type); no shuffle.
//! Metal Defender arms the lasting `NoWeakness` on this Pokémon.
use crate::spec::prelude::*;

const METAL_ENERGY: Pred = Pred::All(&[Pred::BasicEnergy, Pred::Name("Metal Energy")]);

pub static SPEC: CardSpec = CardSpec {
    class: "Archaludonex",
    // Assemble Alloy: when you play this Pokémon from your hand to evolve, you may attach 2 Basic
    // [M] Energy from your discard pile to your [M] Pokémon in any way you like.
    triggers: &[Trigger {
        origin: RuleSource::Ability,
        event: Event::On(EventPred::All(&[EventPred::Kind(EventKind::Evolve), EventPred::This(Role::Card), EventPred::Source(RulesZone::Hand)])),
        steps: &[Step::new(Op::May(MaySpec {
            asker: Who::Me,
            when: Cond::Nonempty(ZoneRef(Who::Me, Zone::Discard), METAL_ENERGY),
            purpose: PromptPurpose::Activate,
            yes: &[Step::new(Op::Attach(AttachSpec {
                target: SlotPred::TypeIs(ct::METAL),
                from: ZoneRef(Who::Me, Zone::Discard),
                predicate: METAL_ENERGY,
                slots: AttachSlots::ActiveBench,
                bounds: Bounds { min: Num::Lit(1), max: Num::Lit(2) },
                ..AttachSpec::DEFAULT
            }))],
            no: &[],
        }))],
    }],
    // Metal Defender: during your opponent's next turn this Pokémon has no Weakness.
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::NoWeakness }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
