//! Genesect ex (BLK 67): Metallic Signal - once during your turn, you may search your deck for up to 2 Evolution [M]
//! Pokémon, reveal them, and put them into your hand. Then, shuffle your deck. Protect Charge - 150; during your
//! opponent's next turn, this Pokémon takes 30 less damage from attacks (after Weakness and Resistance).
//!
//! A search of the hidden deck may find fewer cards than "up to 2", or none (APR D-05, id2301: the choice among cards you
//! can see is 1 to N; a deck you can't see may fail to hold them). The cards are revealed and put into the hand (a
//! PutIntoHand event, cause: this Ability), then the deck is shuffled once (APR E-19); once per turn is the usage marker. "Evolution" is a
//! non-Basic Pokémon that isn't a LV.X. Protect Charge is a lasting damage reduction on this Pokémon until the end of the
//! opponent's next turn.
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
