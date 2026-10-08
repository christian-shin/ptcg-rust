//! Flareon ex (PRE, Tera): Burning Charge — 130; search your deck for up to 2
//! Basic Energy and attach them to 1 of your Pokémon, then shuffle. Carnelian
//! — 280; during your next turn, this Pokémon can't attack. Tera: no damage
//! from attacks while on the Bench.
//!
//! Twinleaf: Burning Charge does nothing with an empty deck. A 0-2 energy
//! choice (no cancel) is followed (only if any were chosen) by a
//! ChoosePokemonPrompt; the cards move deck -> that Pokémon (no reveal). The
//! deck is shuffled afterwards (not reached if the target prompt returned no
//! target).
//!
//! Fixed (phase 4b, R2): Burning Charge also did nothing with no Benched
//! Pokémon, but the text attaches the Energy to 1 of your Pokémon (the Active
//! Flareon ex counts).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Flareonex",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(Op::If(IfSpec { cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any), yes: &[Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::BasicEnergy, bounds: Bounds { min: Num::Lit(0), max: Num::Lit(2) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::AttachToPicked,
                msg: "",
                cancel: false,
                shuffle_first: false,
            })), Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true }))], no: &[] })),
        ] },
        AttackSpec { index: 1, steps: &[
            Step::after_damage(Op::Arm(ArmSpec { what: Lasting::CannotAttackNextTurn })),
        ] },
    ],
    passives: &[
        Passive { origin: RuleSource::CardRule, modifier: Modifier::PreventDamage(PreventDamageSpec { how: PreventHow::Tera, ..PreventDamageSpec::DEFAULT }) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
