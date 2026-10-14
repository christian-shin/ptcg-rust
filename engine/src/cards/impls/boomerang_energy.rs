//! Boomerang Energy (TWM 166): provides [C]. If this card is discarded by an effect of an attack used by the Pokémon it is
//! attached to, attach it from the discard pile to that Pokémon after attacking.
//!
//! Event: its own LeavePlay to the discard pile (an attached card leaving play, D1), caused by an attack whose Pokémon is the
//! one it was attached to (`CauseOnSlot`, read on the spot it left), whatever route discards it (an after-damage removal and
//! Volt Strike's "discard all Energy" included). The trigger records that Pokémon (`Op::Record`); the record stays through a switch. The
//! re-attach runs in the after-attack window, once the attack's effects and the Energy choices they ask are done and before the
//! effects that trigger on the Defending Pokémon (Handheld Fan, id2125); the end of the turn is the fallback for a discard no
//! window followed. It goes back to the Pokémon it was attached to if that Pokémon is still in play, otherwise it stays in the
//! discard pile. Its Energy origin is judged as it was attached there (`trigger::fires_on`): at its own LeavePlay, and afterwards
//! through the record of its Pokémon. The after-attack window is the `AttackDone` event (the attack routine makes it; "after
//! attacking") of its owner's attack.
use crate::spec::prelude::*;

/// Attach this card from the discard pile to the Active Pokémon, once, when it was discarded by
/// an effect of its Pokémon's attack.
const REATTACH: &[Step] = &[Step::new(Op::If(IfSpec {
    cond: Cond::AnySlot(SlotSel::Pokemon(Who::Me), SlotPred::Recorded),
    yes: &[
        // "That Pokémon": the one it was attached to, if it is still in play.
        Step::new(Op::Attach(AttachSpec { from: ZoneRef(Who::Me, Zone::Discard), cards: CardSel::This, onto: Some(SlotExpr::Recorded), ..AttachSpec::DEFAULT })),
        Step::new(Op::Forget(ForgetSpec::Every(Who::Me))),
    ],
    no: &[],
}))];

pub static SPEC: CardSpec = CardSpec {
    class: "BoomerangEnergy",
    triggers: &[
        Trigger {
            origin: RuleSource::Energy,
            event: Event::On(EventPred::All(&[
                EventPred::Kind(EventKind::LeavePlay),
                EventPred::This(Role::Card),
                EventPred::Dest(RulesZone::Discard),
                EventPred::Cause(CausePred::Kind(crate::cause::CauseKind::Attack)),
                EventPred::CauseOnSlot,
            ])),
            // The record stays on the Pokémon through a switch.
            steps: &[Step::new(Op::Record(SlotExpr::Picked))],
        },
        // After the attack's effects (and the Energy choices they ask) are done; the end of the
        // turn is the fallback for a discard no after-attack window followed.
        Trigger { origin: RuleSource::Energy, event: Event::On(EventPred::All(&[EventPred::Kind(EventKind::AttackDone), EventPred::Owner(Who::Me)])), steps: REATTACH },
        Trigger { origin: RuleSource::Energy, event: Event::On(EventPred::All(&[EventPred::Kind(EventKind::EndTurn), EventPred::Owner(Who::Me)])), steps: REATTACH },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();

#[cfg(test)]
mod tests {
    //! No pool attack discards Boomerang Energy and then moves its Pokémon (Mega Latias ex's Strafe
    //! doesn't discard; Mega Dragonite ex's Sky Transport is an Ability), so "that Pokémon" is covered
    //! here: the discard is made by an attack of the Active, the Pokémon then moves or is gone, and the
    //! card follows the Pokémon (or stays in the discard pile).
    use crate::effects::*;
    use crate::game::Game;
    use crate::list::*;
    use crate::state::*;
    use serde_json::json;

    const DURA: &str = "Duraludon PRE 69";
    const BOOM: &str = "Boomerang Energy TWM 166";

    fn run(after: fn(&mut Game, usize)) -> (Game, usize, CardId) {
        let mut names: Vec<&str> = vec![BOOM; 4];
        names.extend([DURA; 4]);
        names.extend(["Metal Energy MEE 8"; 52]);
        let deck: Vec<u16> = names.iter().map(|n| crate::carddb::def_by_full_name(n).unwrap()).collect();
        let mut g = Game::new(7);
        g.start([&deck, &deck]).unwrap();
        g.settle().ok();
        let sc = json!({
            "me": {"reset": true, "active": DURA, "active_energy": [BOOM], "bench": [{"card": DURA}]},
            "opp": {"reset": true, "active": DURA}
        });
        crate::scenario::apply(&mut g, &sc).unwrap();
        let me = g.st.active_player as usize;
        let slot = g.st.players[me].active;
        let card = g.st.slot(me, slot).cards.iter().find(|&c| g.st.cdef(c).name == "Boomerang Energy").unwrap();
        let source = SlotRef::new(me, slot);
        let attack = AttackRef { card: g.st.slot_pokemon(me, slot).unwrap(), index: 0 };
        let cause = crate::cause::Cause::of_attack_at(&g, me as u8, attack, source);
        crate::engine::knockout::leave_play_cards(&mut g, source, &[card], crate::spec::event::RulesZone::Discard, cause, None).unwrap();
        assert!(g.st.players[me].discard.contains(card), "discarded");
        after(&mut g, me);
        g.run_fx(Effect::AfterAttackTriggers { p: me as u8, opp: (1 - me) as u8, attack }).unwrap();
        (g, me, card)
    }

    #[test]
    fn re_attached_to_the_pokemon_it_came_from_after_it_moves() {
        let (g, me, card) = run(|g, me| {
            let bench = g.st.players[me].bench.as_slice()[0];
            let c = crate::engine::change_active::ChangeActiveView::of(g, me, Some(bench), crate::spec::event::ActiveChange::Retreat, crate::cause::Cause::rule(crate::cause::RuleWhich::Retreat, me as u8));
            assert!(crate::engine::change_active::change_active(g, c).unwrap());
        });
        let on_bench = g.st.players[me].bench.as_slice().iter().any(|&s| g.st.slot(me, s).cards.contains(card));
        assert!(on_bench, "back on the Pokémon that is now Benched");
        assert!(!g.st.slot(me, g.st.players[me].active).cards.contains(card));
    }

    #[test]
    fn nothing_when_that_pokemon_is_gone() {
        let (g, me, card) = run(|g, me| {
            let a = g.st.players[me].active;
            g.st.players[me].slots[a as usize].cards = Default::default();
            });
        assert!(g.st.players[me].discard.contains(card), "stays in the discard pile");
    }
}
