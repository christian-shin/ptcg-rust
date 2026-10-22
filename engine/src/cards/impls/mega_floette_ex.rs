//! Mega Floette ex (CRI 35): Gentle Light - heal 30 damage from each Pokémon (both yours and your opponent's). Eternity Bloom - 200;
//! search your deck for up to 4 Basic [P] Energy cards and attach them to your Benched Pokémon in any way you like. Then, shuffle
//! your deck.
//!
//! Gentle Light: a Heal event of 30 on each of your Pokémon, then on each of your opponent's (Cresselia SFA's Healing Pirouette).
//! Eternity Bloom: one search (0 to 4, an Attach event per card onto the Bench) and the shuffle (Cinderace MEG's Turbo Flare,
//! APR E-19); no Bench, no search.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegaFloetteEx",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::ForEach(ForEachSpec { over: SlotSel::Pokemon(Who::Me), body: &[Step::new(Op::Heal(HealSpec { target: SlotTarget::Slot(SlotExpr::Picked), hp: Num::Lit(30), clear_conditions: false }))] })),
                Step::after_damage(Op::ForEach(ForEachSpec { over: SlotSel::Pokemon(Who::Opp), body: &[Step::new(Op::Heal(HealSpec { target: SlotTarget::Slot(SlotExpr::Picked), hp: Num::Lit(30), clear_conditions: false }))] })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[Step::after_damage(Op::If(IfSpec {
                cond: Cond::All(&[Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any), Cond::Cmp(Num::BenchCount(Who::Me), CmpOp::Gt, Num::Lit(0))]),
                yes: &[
                    Step::new(Op::Attach(AttachSpec {
                        from: ZoneRef(Who::Me, Zone::Deck),
                        predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Psychic Energy")]),
                        slots: AttachSlots::Bench,
                        bounds: Bounds { min: Num::Lit(0), max: Num::Lit(4) },
                        ..AttachSpec::DEFAULT
                    })),
                    Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
                ],
                no: &[],
            }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
