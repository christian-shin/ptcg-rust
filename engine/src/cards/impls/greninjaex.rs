//! Greninja ex (TWM, Tera): Shinobi Blade — 170; you may search your deck for a card and put it into your hand, then
//! shuffle. Mirage Barrage — discard 2 Energy from this Pokémon; 120 damage to 2 of your opponent's Pokémon.
//!
//! Shinobi Blade skips the search on an empty deck; otherwise a yes / no, then 1 card to the hand (a searcher must take
//! a card, id1980) and a shuffle.
//! Mirage Barrage: 2 targets (min = max = min(2, the opponent's Pokémon in play)), each a Damage event caused by the
//! attack (Weakness and Resistance only for the Active Pokémon, APR B-08), and the Energy is discarded after the
//! damage (id2002: a Double Turbo Energy discarded for it still reduces the damage). The Tera rule is
//! `TERA_RULE`: a `Prevent` over `Kind(Damage)` on this Pokémon while it is on the Bench.
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
            })), Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true }))], no: &[] }))], no: &[] })),
        ] },
        AttackSpec { index: 1, steps: &[
            Step::after_damage(Op::EachSlot(EachSlotSpec { among: SlotSel::Pokemon(Who::Opp), choose: Some(ChooseN { chooser: Who::Me, min: Num::Min(&Num::Lit(2), &Num::SlotCount(SlotSel::Pokemon(Who::Opp), SlotPred::Any)), max: Num::Min(&Num::Lit(2), &Num::SlotCount(SlotSel::Pokemon(Who::Opp), SlotPred::Any)), msg: "CHOOSE_POKEMON_TO_DAMAGE" }), what: EachWhat::Damage(DamageCalc::Auto), amount: Num::Lit(120), ..EachSlotSpec::DEFAULT })),
            Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(SlotExpr::Active(Who::Me)), selection: EnergySelection::Choose { count: 2, ty: ct::COLORLESS }, ..DiscardEnergySpec::DEFAULT })),
        ] },
    ],
    passives: &[
        Passive { origin: RuleSource::CardRule, modifier: Modifier::Prevent(TERA_RULE) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
