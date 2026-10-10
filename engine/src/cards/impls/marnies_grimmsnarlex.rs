//! Marnie's Grimmsnarl ex (DRI 136 / ASC 287): Punk Up — when you play this Pokémon from your hand to evolve 1 of your
//! Pokémon during your turn, you may search your deck for up to 5 Basic [D] Energy cards and attach them to your
//! Marnie's Pokémon in any way you like, then shuffle. Shadow Bullet — 180; this attack also does 30 damage to 1 of your
//! opponent's Benched Pokémon.
//!
//! Punk Up is an `On(Evolve & This(Card) & Source(Hand))` trigger with an Ability origin (Attach events from the deck).
//! Shadow Bullet's 30 is the attack's damage to a Benched Pokémon (one Damage event, `DamageCalc::Put`: no Weakness or
//! Resistance), not damage counters: Battle Cage doesn't stop it (JP FAQ マリィのオーロンゲex / バトルコロシアム
//! 「はい、できます。」).
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
        event: Event::On(EventPred::All(&[EventPred::Kind(EventKind::Evolve), EventPred::This(Role::Card), EventPred::Source(RulesZone::Hand)])),
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
