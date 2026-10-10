//! Boomerang Energy (TWM): provides [C]. If discarded by an effect of an
//! attack of the Pokémon it is attached to, attach it from the discard pile
//! to that Pokémon after attacking.
//!
//! Fixed (phase 4 wave 2, B-PC-7): it is re-attached to the Pokémon it was attached to (the marker
//! is kept on that Pokémon), if that Pokémon is still in play; nothing otherwise.
//!
//! Fixed (phase 4b, R7F-10; ruling 1650): it was re-attached at EndTurn; it is
//! now re-attached in AfterAttackTriggersEffect, once the attack's effects and the
//! Energy choices they ask after the damage are done (EndTurn stays as the fallback
//! for a discard no AfterAttackTriggersEffect followed), before the effects that trigger
//! on the Defending Pokémon (Handheld Fan) resolve: AfterAttackEffect reaches
//! Pokémon, then Energy, then Trainers.
//!
//! Fixed (phase 4b, W4): the re-attach was armed by a marker set on the
//! AttackEffect, but the attacker's own handler runs before its attached
//! Energy sees that effect, so a discard made synchronously by the attack
//! (Volt Strike, "discard all Energy") happened before the marker existed and
//! the card was never re-attached.
//!
//! Events batch 7: "if this card is discarded by an effect of an attack of the Pokémon it is attached to" is a trigger
//! over its own LeavePlay to the discard pile (an attached card leaving play, user decision D1), caused by an attack whose
//! Pokémon is the one it was attached to (`CauseOnSlot`, read on the spot it left), whatever route discards it (an
//! after-damage removal included). Its Energy origin is judged as it was attached there (`trigger::fires_on`).
use crate::spec::prelude::*;

const DISCARDED: &str = "BOOMERANG_DISCARDED_MARKER";

/// Attach this card from the discard pile to the Active Pokémon, once, when it was discarded by
/// an effect of its Pokémon's attack.
const REATTACH: &[Step] = &[Step::new(Op::If(IfSpec {
    cond: Cond::AnySlot(SlotSel::Pokemon(Who::Me), SlotPred::MarkerFromThis(DISCARDED)),
    yes: &[
        // "That Pokémon": the one it was attached to, if it is still in play.
        Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Discard), cards: CardSel::This, place: Place::AttachTo(SlotExpr::Marked(DISCARDED)), ..MoveSpec::DEFAULT })),
        Step::new(Op::ClearMarker(ClearMarkerSpec { scope: MarkerScope::EveryPokemon(Who::Me), name: DISCARDED, from: MarkerFrom::This })),
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
            // The marker stays on the Pokémon through a switch (a TrainerEffect source marker is kept).
            steps: &[Step::new(Op::SetMarker(SetMarkerSpec { scope: MarkerScope::Slot(SlotExpr::Picked), name: DISCARDED, source: RuleSource::TrainerEffect }))],
        },
        // After the attack's effects (and the Energy choices they ask) are done; the end of the
        // turn is the fallback for a discard no after-attack window followed.
        Trigger { origin: RuleSource::Energy, event: Event::OnAfterAttackTriggers(OnAfterAttackTriggersSpec {}), steps: REATTACH },
        Trigger { origin: RuleSource::Energy, event: Event::OnEndTurn(OnEndTurnSpec { whose: Turn::Owner }), steps: REATTACH },
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
            crate::list::touch();
        });
        assert!(g.st.players[me].discard.contains(card), "stays in the discard pile");
    }
}
