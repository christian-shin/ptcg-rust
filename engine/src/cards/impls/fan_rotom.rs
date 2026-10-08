//! Fan Rotom (SCR): Fan Call — once during your first turn, search your deck
//! for up to 3 [C] Pokémon with 100 HP or less, reveal them, and put them
//! into your hand, then shuffle (1 Fan Call per turn). Assault Landing —
//! 70; does nothing if there is no Stadium in play.
//!
//! Twinleaf: `player.usedFanCall` is set as soon as the Ability is used (phase
//! 4b: it used to be set only when cards were taken, so the Ability could be
//! used again after taking none) and cleared at every end of turn; "first turn"
//! is `state.turn <= 2`. ABILITY_USED is set in the choose callback; the
//! ShuffleDeckPrompt follows the (non-yielding) ShowCardsPrompt with no wait.
//! Assault Landing sets the damage to exactly 0 or 70.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "FanRotom",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::before_damage(damage_is(Num::If(&Cond::StadiumInPlay(Pred::Any), &Num::Lit(70), &Num::Lit(0)))),
        ] },
    ],
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurnShared("FAN_CALL_MARKER"),
        needs: &[Cond::Cmp(Num::Turn, CmpOp::Le, Num::Lit(2)), Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any)],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::All(&[Pred::Pokemon, Pred::PokemonType(ct::COLORLESS), Pred::HpAtMost(100)]), bounds: Bounds { min: Num::Lit(0), max: Num::Lit(3) }, caps: &[Cap { kind: CapKind::Pokemon, max: Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Deck), Pred::All(&[Pred::Pokemon, Pred::PokemonType(ct::COLORLESS), Pred::HpAtMost(100)])), &Num::Lit(3)) }], ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: true },
                msg: "",
                cancel: false,
                shuffle_first: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
