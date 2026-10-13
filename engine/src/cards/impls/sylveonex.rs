//! Sylveon ex (SSP, Tera): Magical Charm - 160; during your opponent's next turn, the Defending Pokémon's attacks do 100 less damage.
//! Angelite - choose 2 of your opponent's Benched Pokémon; they shuffle those Pokémon and all attached cards into their deck; can't be
//! used if 1 of your Pokémon used Angelite during your last turn. Tera: no attack damage while on the Bench.
//!
//! Angelite does nothing with an empty opposing Bench; each chosen Pokémon is moved to the opponent's deck and followed by its own
//! shuffle for the opponent (no trailing wait). "Choose 2" is exactly 2 (1 for a single Benched Pokémon only); using the attack counts
//! even when it does nothing.
//!
//! "If 1 of your Pokémon used Angelite during your last turn" reads the attack the player used during their previous turn
//! (`Cond::UsedAttackLastTurn`), whichever Pokémon used it, a copycat too (id2402); no marker is kept, so a copied Angelite or a
//! Knocked Out Sylveon ex doesn't leave one behind.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Sylveonex",
    passives: &[Passive { origin: RuleSource::CardRule, modifier: Modifier::Prevent(TERA_RULE) }],
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::DealsLessDamage(100) }))] },
        AttackSpec {
            index: 1,
            steps: &[
                // Can't be used if 1 of your Pokémon used Angelite during your last turn.
                Step::before_damage(Op::Fail(FailSpec { unless: Cond::Not(&Cond::UsedAttackLastTurn("Angelite")), error: "BLOCKED_BY_EFFECT" })),
                // Choose 2 of your opponent's Benched Pokémon; they shuffle those Pokémon and all attached cards into their deck.
                Step::after_damage(Op::EachSlot(EachSlotSpec {
                    among: SlotSel::Bench(Who::Opp),
                    choose: Some(ChooseN { chooser: Who::Me, min: Num::Min(&Num::Lit(2), &Num::BenchCount(Who::Opp)), max: Num::Lit(2), msg: "CHOOSE_POKEMON_TO_DAMAGE" }),
                    what: EachWhat::ShuffleIntoDeck,
                    ..EachSlotSpec::DEFAULT
                })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
