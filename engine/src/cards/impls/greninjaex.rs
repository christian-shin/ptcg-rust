//! Greninja ex (TWM, Tera): Shinobi Blade — 170; you may search your deck
//! for a card and put it into your hand, then shuffle. Mirage Barrage —
//! discard 2 Energy from this Pokémon; 120 damage to 2 of your opponent's
//! Pokémon.
//!
//! Fixed (phase 4b): Shinobi Blade skips the search on an empty deck (it
//! threw CANNOT_USE_POWER, making the attack unusable). Otherwise a Confirm (SEARCH_DECK_FOR_CARD); yes → ChooseCardsPrompt (min 1, max 1, no
//! filter) → MOVE_CARDS deck→hand → bare ShuffleDeckPrompt. Mirage Barrage:
//! ChooseEnergyPrompt ([C][C] over the Active's energy map) → ChoosePokemon
//! (opponent, Active/Bench, min = max = min(2, the opponent's Pokémon in play);
//! phase 4b: min used to be 1) → DAMAGE_OPPONENT_POKEMON(120) and
//! only then the DiscardCardsEffect of the chosen energy (target Active).
//! The Tera rule prevents PutDamageEffects on this Pokémon on the Bench.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Greninjaex",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(Op::If(IfSpec { cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any), yes: &[Step::new(Op::May(MaySpec { asker: Who::Me, when: Cond::True, msg: "SEARCH_DECK_FOR_CARD", yes: &[Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::Any, bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: false },
                msg: "",
                cancel: false,
                shuffle_first: false,
            })), Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck) }))], no: &[] }))], no: &[] })),
        ] },
        AttackSpec { index: 1, steps: &[
            Step::after_damage(Op::DamageChosen(DamageChosenSpec { among: SlotSel::Pokemon(Who::Opp), count: 2, hp: Num::Lit(120), calc: DamageCalc::Auto, msg: "CHOOSE_POKEMON_TO_DAMAGE" })),
            Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotExpr::Active(Who::Me), selection: EnergySelection::Units(2) })),
        ] },
    ],
    passives: &[
        Passive { origin: RuleSource::CardRule, modifier: Modifier::PreventDamage(PreventDamageSpec { how: PreventHow::Tera, ..PreventDamageSpec::DEFAULT }) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
