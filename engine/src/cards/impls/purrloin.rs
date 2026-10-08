//! Purrloin (WHT): Invite Evil — search your deck for up to 3 [D] Pokémon,
//! reveal them, and put them into your hand; shuffle.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Purrloin",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::If(IfSpec {
            cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any),
            yes: &[Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::All(&[Pred::Pokemon, Pred::PokemonType(ct::DARK)]), bounds: Bounds { min: Num::Lit(0), max: Num::Lit(3) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: true },
                msg: "",
                cancel: true,
                shuffle_first: false,
            })), Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true }))],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
