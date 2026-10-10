//! Alakazam (MEG 56): Psychic Draw — once during your turn, when you play this Pokémon from your hand to evolve 1 of
//! your Pokémon, you may draw 3 cards. Powerful Hand — place 2 damage counters on your opponent's Active Pokémon for
//! each card in your hand.
//!
//! Psychic Draw is an `On(Evolve & This(Card) & Source(Hand))` trigger with an Ability origin (an Ability lock turns it
//! off). Powerful Hand is one PlaceCounters event by the attack (an effect, not damage: no Weakness or Resistance;
//! Hide 'n' Sneak prevents it, JP FAQ フーディン / チャデス 「いいえ、できません。」; Battle Cage doesn't on the Active
//! Pokémon, JP FAQ フーディン / バトルコロシアム 「はい、できます。」).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Alakazam@MEG",
    // Powerful Hand: 2 damage counters on the opponent's Active for each card in your hand (one PlaceCounters event).
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::PlaceCounters(PlaceCountersSpec {
            target: SlotTarget::Slot(OPP_ACTIVE),
            counters: Num::Mul(&Num::Lit(2), &Num::ZoneSize(ZoneRef(Who::Me, Zone::Hand)))
        }))],
    }],
    // Psychic Draw: when you play this Pokémon from your hand to evolve, you may draw 3 cards.
    triggers: &[Trigger {
        origin: RuleSource::Ability,
        event: Event::On(EventPred::All(&[EventPred::Kind(EventKind::Evolve), EventPred::This(Role::Card), EventPred::Source(RulesZone::Hand)])),
        steps: &[Step::new(Op::May(MaySpec {
            asker: Who::Me,
            when: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any),
            msg: "WANT_TO_USE_ABILITY",
            yes: &[Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(3)) }))],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
