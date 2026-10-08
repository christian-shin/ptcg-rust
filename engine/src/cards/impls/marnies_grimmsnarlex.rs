//! Marnie's Grimmsnarl ex (DRI): Punk Up — when you evolve into this Pokémon
//! from your hand, you may search your deck for up to 5 Basic [D] Energy and
//! attach them to your Marnie's Pokémon, then shuffle. Shadow Bullet — 180,
//! and 30 damage to 1 of the opponent's Benched Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MarniesGrimmsnarlex",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::DamageSlot(DamageSlotSpec {
            target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Bench(Who::Opp), msg: "CHOOSE_POKEMON_TO_DAMAGE" }),
            hp: Num::Lit(30),
            target_damage_mul: 0,
            calc: DamageCalc::Put,
            when: Cond::True,
        }))],
    }],
    triggers: &[Trigger {
        origin: RuleSource::Ability,
        event: Event::OnEnterPlay(OnEnterPlaySpec { method: EnterMethod::Evolve }),
        steps: &[Step::new(Op::If(IfSpec {
            cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any),
            yes: &[Step::new(Op::May(MaySpec {
                asker: Who::Me,
                when: Cond::True,
                msg: "WANT_TO_USE_ABILITY",
                yes: &[
                    Step::new(Op::Attach(AttachSpec {
                        from: ZoneRef(Who::Me, Zone::Deck),
                        predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Darkness Energy")]),
                        slots: AttachSlots::BenchActive,
                        target: Pred::Tag(crate::types::tag::MARNIES),
                        bounds: Bounds { min: Num::Lit(0), max: Num::Lit(5) },
                        cancel: true,
                        ..AttachSpec::DEFAULT
                    })),
                    Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
                ],
                no: &[],
            }))],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
