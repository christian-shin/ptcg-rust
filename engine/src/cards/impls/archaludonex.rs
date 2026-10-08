//! Archaludon ex (SSP 130): Assemble Alloy — when you play this Pokémon from
//! your hand to evolve, you may attach 2 Basic [M] Energy from your discard
//! pile to your [M] Pokémon in any way you like. Metal Defender — 220;
//! during your opponent's next turn this Pokémon has no Weakness.
//!
//! Twinleaf: fires on any EvolveEffect for this card (Rare Candy included)
//! when the discard holds an Energy named "Metal Energy"; blockedTo (non-[M]
//! printed types) is taken before the evolution; the ability-lock probe runs
//! after it. AttachEnergyPrompt from the discard (min 1 since phase 4b R7E: "up to
//! 2" in an Ability takes at least 1, rulings 1853/1778; it was min 0; max 2,
//! no cancel),
//! then one MOVE_CARDS per transfer; no shuffle. Metal Defender reduces a
//! ThisPokemonHasNoWeaknessDuringOpponentsNextTurnEffect.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Archaludonex",
    // Assemble Alloy: when you play this Pokémon from your hand to evolve, you may attach 2 Basic
    // [M] Energy from your discard pile to your [M] Pokémon in any way you like.
    triggers: &[Trigger {
        origin: RuleSource::Ability,
        event: Event::OnEnterPlay(OnEnterPlaySpec { method: EnterMethod::Evolve }),
        steps: &[Step::new(Op::May(MaySpec {
            asker: Who::Me,
            when: Cond::Nonempty(ZoneRef(Who::Me, Zone::Discard), Pred::All(&[Pred::Energy, Pred::Name("Metal Energy")])),
            msg: "WANT_TO_USE_ABILITY",
            yes: &[Step::new(Op::Attach(AttachSpec {
                from: ZoneRef(Who::Me, Zone::Discard),
                predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Metal Energy")]),
                slots: AttachSlots::ActiveBench,
                target: Pred::PrintedType(ct::METAL),
                bounds: Bounds { min: Num::Lit(1), max: Num::Lit(2) },
                route: AttachRoute::Move,
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
