//! Heliolisk (ASC 64): Frilled Generator - once during your turn, if you played Canari from your hand this turn, you may
//! use this Ability. Search your deck for up to 2 Basic [L] Energy cards and attach them to this Pokémon. Then, shuffle
//! your deck. Powerful Bolt - 70×; flip a coin for each Energy attached to this Pokémon. This attack does 70 damage for
//! each heads.
//!
//! Frilled Generator needs a Supporter named Canari played from the hand this turn (`Cond::PlayedThisTurn`, E-26; each copy
//! of the Ability can be used, id2284); the search-and-attach is Deoxys CRI 31's Genome Charge for Basic [L] Energy, then
//! the shuffle (APR E-19). Powerful Bolt flips one coin per Energy this Pokémon provides (id2237) before the damage and the
//! damage is 70 per heads.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "HelioliskASC",
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn,
        needs: &[Cond::PlayedThisTurn(Who::Me, Pred::Name("Canari"))],
        steps: &[Step::new(Op::If(IfSpec {
            cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any),
            yes: &[
                Step::new(Op::Pick(PickSpec {
                    from: ZoneRef(Who::Me, Zone::Deck),
                    predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Lightning Energy")]),
                    bounds: Bounds { min: Num::Lit(0), max: Num::Lit(2) },
                    into: 0,
                    ..PickSpec::DEFAULT
                })),
                Step::new(Op::If(IfSpec {
                    cond: Cond::Chosen(0),
                    yes: &[Step::new(Op::Attach(AttachSpec { from: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::Chosen(0), onto: Some(SlotExpr::This), ..AttachSpec::DEFAULT }))],
                    no: &[],
                })),
                Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
            ],
            no: &[],
        }))],
    }],
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(Op::Coin(CoinSpec {
            flips: Flips::Count(Num::EnergyOn(SlotSel::One(MY_ACTIVE), EnergyUnit::ProvidedUnits)),
            per_heads: PerHeads::DamageIs(70),
            ..CoinSpec::DEFAULT
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
