//! Spiritomb (M5): Spiritual End — if you have 13 or more Pokémon with the
//! Hide 'n' Sneak Ability in your discard pile, choose 2 of your opponent's
//! Pokémon and quadruple the number of damage counters on each of them.
//!
//! Twinleaf: attack damage is zeroed first; the ChoosePokemonPrompt needs
//! exactly min(2, the opponent's Pokémon in play) targets (phase 4b: it needed
//! 2 even with a lone opposing Pokémon, a prompt with no valid answer); each
//! chosen Pokémon with damage gets one
//! PlaceDamageCountersEffect (source = this card) adding 3x its damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Spiritomb",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::before_damage(damage_is(Num::Lit(0))),
            Step::after_damage(Op::If(IfSpec {
                cond: Cond::Cmp(Num::CardCount(ZoneRef(Who::Me, Zone::Discard), Pred::HasAbilityNamed("Hide 'n' Sneak")), CmpOp::Ge, Num::Lit(13)),
                // Choose 2 of the opponent's Pokémon and quadruple the damage counters on each:
                // 3 times what is there is added.
                yes: &[Step::new(Op::EachSlot(EachSlotSpec {
                    among: SlotSel::Pokemon(Who::Opp),
                    choose: Some(ChooseN { chooser: Who::Me, min: Num::Min(&Num::Lit(2), &Num::SlotCount(SlotSel::Pokemon(Who::Opp), SlotPred::Any)), max: Num::Min(&Num::Lit(2), &Num::SlotCount(SlotSel::Pokemon(Who::Opp), SlotPred::Any)), msg: "CHOOSE_POKEMON_TO_DAMAGE" }),
                    what: EachWhat::Counters,
                    per_damage: 3,
                    only_damaged: true,
                    ..EachSlotSpec::DEFAULT
                }))],
                no: &[],
            })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
