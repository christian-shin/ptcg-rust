//! Iron Thorns ex (PRE 32 / TWM 77): Initialization - as long as this Pokémon is in the Active Spot, Pokémon with a Rule Box in
//! play (both yours and your opponent's) have no Abilities, except for Future Pokémon. Volt Cyclone - 140; move an Energy from
//! this Pokémon to 1 of your Benched Pokémon.
//!
//! Initialization is an `ActiveLockSpec`, an Ability lock (ENGINE section 7: the has-no-Ability lock pass) over every
//! Pokémon in play, whoever owns it, that has a Rule Box (`locked`) and isn't a Future Pokémon (`except`), while this Pokémon is
//! the Active Pokémon of its owner (`while_`: either player's) and its own Ability works. A Pokémon being put into play or evolved from the hand is already covered (scenarios
//! fix-iron-thorns-ex-hand-lock-*). An Ability it turns off is not an Ability the Pokémon "has" (JP FAQ Iron Thorns ex: Cofagrigus's
//! counters). Volt Cyclone moves one Energy from this Pokémon to a Benched Pokémon after the damage (a MoveEnergy event, id1653),
//! with no cancel and only when there is a Benched Pokémon.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "IronThornsex",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(Op::If(IfSpec { cond: Cond::AnySlot(SlotSel::Bench(Who::Me), SlotPred::Any), yes: &[Step::new(Op::Attach(AttachSpec { from: ZoneRef(Who::Me, Zone::Attached(SlotExpr::Active(Who::Me))), predicate: Pred::Energy, bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, ..AttachSpec::DEFAULT }))], no: &[] })),
        ] },
    ],
    passives: &[
        Passive {
            origin: RuleSource::Ability,
            modifier: Modifier::ActiveLock(ActiveLockSpec {
                // "Pokémon with a Rule Box in play (both yours and your opponent's) have no Abilities, except for Future Pokémon."
                locked: SlotPred::Top(Pred::RuleBox),
                except: Pred::Tag(crate::types::tag::FUTURE),
                // "As long as this Pokémon is in the Active Spot".
                while_: &[LockWhile::Active],
            }),
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
