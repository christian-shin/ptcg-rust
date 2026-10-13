//! Flutter Mane (PRE 43 / TEF 78): Midnight Fluttering — as long as this
//! Pokémon is in the Active Spot, your opponent's Active Pokémon has no Abilities,
//! except for Midnight Fluttering. Hex Hurl — 90; put 2 damage counters on your
//! opponent's Benched Pokémon in any way you like.
//!
//! Rule: Midnight Fluttering is an `ActiveLockSpec`: an Ability lock on the opponent's
//! Active Pokémon (`locked`), except the Pokémon that has the Ability Midnight Fluttering
//! (`except`), while this Pokémon is its owner's Active (`while_`; section 7; an
//! Ability named "Hide 'n' Sneak" takes precedence, ruling 1877, whichever came
//! into play first). Hex Hurl is a SpreadDamage of counters on the opponent's
//! Bench: PlaceCounters events (cause: this attack, no Weakness or Resistance,
//! APR C-07) that Hide 'n' Sneak, Mist Energy and Battle Cage refuse one by one.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "FlutterMane",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(Op::SpreadDamage(SpreadDamageSpec { chooser: Who::Me, side: Who::Opp, slots: SpreadSlots::Bench, total_hp: 20, unit_hp: 10, cap_bonus_hp: None, apply: SpreadApply::Counters })),
        ] },
    ],
    passives: &[
        Passive {
            origin: RuleSource::Ability,
            modifier: Modifier::ActiveLock(ActiveLockSpec {
                // "Your opponent's Active Pokémon has no Abilities, except for Midnight Fluttering."
                locked: SlotPred::All(&[SlotPred::Not(&SlotPred::OnMySide), SlotPred::IsActive]),
                except: Pred::HasAbilityNamed("Midnight Fluttering"),
                // "As long as this Pokémon is in the Active Spot".
                while_: &[LockWhile::Active],
            }),
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
