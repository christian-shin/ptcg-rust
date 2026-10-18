//! Named table rules: options a host may set when it creates a game, for the few places where another engine reads the
//! rules another way. Every option is off by default, and off is what the rulings say (docs/rulings/RULES.md); the
//! engine's golden corpus and scenarios run with every option off.
//!
//! An option is one rules concept, named after it, read where that concept is decided (a plain field read, no lookup). The
//! options are set before the game starts ([`crate::game::Game::new`], then `g.st.rules`; `ptcg::host::StartOptions::rules`;
//! the Python `Env(..., rules={...})`; a scenario's `"rules": {...}`) and never change during a game, so a fork or a
//! copy of the game carries them with its state and no search can see two values: they are not part of the state hash.

/// The table rules of a game (see the module documentation). `TableRules::default()` is the rulings.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TableRules {
    /// A cost paid by discarding Energy cards that can be paid in only one way (every Energy attached, or copies of one card
    /// providing the same) is asked anyway.
    ///
    /// Off (the rulings): which of several copies of one card is discarded can't change the game, so nothing is asked
    /// (RULES.md "Options that can't do anything", user decision 2; "Paying a cost with different cards is the player's
    /// choice", APR A-03 step 2: different cards are asked). The engine pays without a question only a Retreat Cost
    /// (`engine::retreat`); every other Energy payment always asks.
    pub ask_single_way_payments: bool,
    /// An attack that can do nothing (no damage, and no effect of its text could change the game, or a choice its text
    /// starts with has nothing to choose: the usability walk of Trainers and Abilities, `spec::run::usable_by_rules`)
    /// can't be used: it isn't offered.
    ///
    /// Off (the rulings): an attack whose Energy cost is paid may be declared even when its effect can't be done; it does
    /// nothing and the turn ends (APR A-01: an attack needs its Energy; RULES.md "Options that can't do anything" is about
    /// questions and the uses of Trainers and Abilities, not attacks).
    pub hide_attacks_that_can_do_nothing: bool,
    /// The Prize cards for a Knock Out that leaves the opponent no Pokémon in play are chosen (the Prize question), as
    /// for any other Knock Out, before the game ends.
    ///
    /// Off (the rulings): once the opponent has no Pokémon in play the game is decided (APR E: the winner is determined
    /// after every effect; `knockout::check_winner`), so which face-down Prize cards are taken can't change it: they are
    /// taken without a question (RULES.md "Options that can't do anything", user decision 2). The winner is the same
    /// either way. Taking every Prize card left is never asked.
    pub prizes_chosen_when_opponent_has_no_pokemon: bool,
    /// A card a player looks at from a hidden zone of the opponent (the top of the opponent's deck) is shown to its owner
    /// too: the game record's Look entry and `TableView::looking` count the owner among those who see it.
    ///
    /// Off (the rulings): the player looking sees the cards; their owner sees them only if they are the one looking
    /// (RULES.md "Who sees a card set aside while a player looks at it", batch 10 E3 / E7; `record::look_seen_by`). A
    /// card from a public zone is seen by everyone either way.
    pub looked_cards_shown_to_owner: bool,
    /// An Item or Supporter card that evolves a Pokémon (Rare Candy) finishes first: the card goes to the discard pile,
    /// then the effects the evolution triggers ("when you play this Pokémon from your hand to evolve 1 of your Pokémon",
    /// Kadabra's Psychic Draw) apply (`engine::order`'s held window, in the order they fired).
    ///
    /// Off (the rulings): a triggered effect applies right after the event that triggers it (events design: an `Event::On`
    /// trigger runs once its event is done; RULES.md "On-play Abilities": it triggers once the card is played and on the
    /// board), so the Ability is offered while the Trainer resolves, before the card is discarded.
    pub evolving_trainer_discarded_before_evolve_triggers: bool,
}

impl TableRules {
    /// Every option's name, in the order of the fields.
    pub const NAMES: &'static [&'static str] = &[
        "ask_single_way_payments",
        "hide_attacks_that_can_do_nothing",
        "prizes_chosen_when_opponent_has_no_pokemon",
        "looked_cards_shown_to_owner",
        "evolving_trainer_discarded_before_evolve_triggers",
    ];

    /// Set the option `name` (one of [`TableRules::NAMES`]); an unknown name is an error.
    pub fn set(&mut self, name: &str, on: bool) -> Result<(), String> {
        let field = match name {
            "ask_single_way_payments" => &mut self.ask_single_way_payments,
            "hide_attacks_that_can_do_nothing" => &mut self.hide_attacks_that_can_do_nothing,
            "prizes_chosen_when_opponent_has_no_pokemon" => &mut self.prizes_chosen_when_opponent_has_no_pokemon,
            "looked_cards_shown_to_owner" => &mut self.looked_cards_shown_to_owner,
            "evolving_trainer_discarded_before_evolve_triggers" => &mut self.evolving_trainer_discarded_before_evolve_triggers,
            _ => return Err(format!("unknown table rule {:?} (known: {})", name, Self::NAMES.join(", "))),
        };
        *field = on;
        Ok(())
    }

    /// The options of a JSON object of names to booleans (a scenario's `"rules"`); an unknown name or a value that isn't a
    /// boolean is an error.
    pub fn from_json(v: &serde_json::Value) -> Result<TableRules, String> {
        let mut r = TableRules::default();
        let o = v.as_object().ok_or("rules: not an object of names to booleans")?;
        for (k, x) in o {
            let on = x.as_bool().ok_or_else(|| format!("rules: {:?} is not a boolean", k))?;
            r.set(k, on)?;
        }
        Ok(r)
    }

    /// Every option on (a smoke test's game).
    pub fn all_on() -> TableRules {
        let mut r = TableRules::default();
        for n in Self::NAMES {
            r.set(n, true).expect("a known name");
        }
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_name_sets_its_own_option_and_unknown_names_are_refused() {
        let all = TableRules::all_on();
        for n in TableRules::NAMES {
            let mut r = TableRules::default();
            r.set(n, true).unwrap();
            assert_ne!(r, TableRules::default(), "{n} sets nothing");
            r.set(n, false).unwrap();
            assert_eq!(r, TableRules::default(), "{n} doesn't clear");
        }
        assert_ne!(all, TableRules::default());
        assert!(TableRules::default().set("no_such_rule", true).is_err());
        assert!(TableRules::from_json(&serde_json::json!({"no_such_rule": true})).is_err());
        assert!(TableRules::from_json(&serde_json::json!({"ask_single_way_payments": 1})).is_err());
        assert_eq!(TableRules::from_json(&serde_json::json!({"ask_single_way_payments": true})).unwrap().ask_single_way_payments, true);
    }
}
