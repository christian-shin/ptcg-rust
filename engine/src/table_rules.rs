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
}

impl TableRules {
    /// Every option's name, in the order of the fields.
    pub const NAMES: &'static [&'static str] = &["ask_single_way_payments"];

    /// Set the option `name` (one of [`TableRules::NAMES`]); an unknown name is an error.
    pub fn set(&mut self, name: &str, on: bool) -> Result<(), String> {
        let field = match name {
            "ask_single_way_payments" => &mut self.ask_single_way_payments,
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
