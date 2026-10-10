//! Genesect ex (BLK / SV11B): Metal Signal — once during your turn, search
//! your deck for up to 2 [M] Evolution Pokémon, reveal them, put them into
//! your hand, then shuffle. Protect Charge — 150; during your opponent's next
//! turn this Pokémon takes 30 less damage from attacks.
//!
//! Twinleaf: the once-per-turn marker lives on the player (per card); the
//! blocked indices are computed on the unsorted deck ("Evolution" = non-empty
//! `evolvesFrom`, not LV.X). Protect Charge writes `damageReductionNextTurn`
//! on the player's Active (whatever it is).
use crate::spec::prelude::*;
use crate::types::Stage;

pub static SPEC: CardSpec = CardSpec {
    class: "Genesectex",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(Op::Arm(ArmSpec { what: Lasting::TakesLessDamage(30) })),
        ] },
    ],
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn("METAL_SIGNAL_MARKER"),
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::All(&[Pred::Pokemon, Pred::PokemonType(ct::METAL), Pred::Not(&Pred::Basic), Pred::Not(&Pred::StageIs(Stage::LvX))]), bounds: Bounds { min: Num::Lit(0), max: Num::Lit(2) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: true },
                msg: "",
                cancel: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
