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
    /// An attack that can do nothing can't be used: it isn't offered. It can do nothing when it has no damage and its text
    /// has nothing to act on: a choice it starts with has nothing to choose, or an effect it does has no object (cards to
    /// move from a known zone and a number of them above 0, a card in a deck to search or draw, room on the Bench, an
    /// Energy attached to move, a Pokémon to choose, an attack to use as this attack). "Up to" doesn't change it. A
    /// condition the text reads before it does anything ("If you have 13 or more ...", "If your opponent's Active Pokémon
    /// has exactly 6 damage counters on it") is read as the board is now, and the effects it doesn't allow have nothing to
    /// act on; a condition on which cards a deck holds isn't read (the deck is hidden). An effect on a Pokémon the text
    /// names has its object while that Pokémon is in play, whether or not the effect would change it (Hold Still's "Heal
    /// 10 damage from this Pokémon" at full HP is offered); a search has something to act on while the deck holds a card
    /// (which cards, and the copies of a named card in known areas, aren't read). The usability walk of Trainers and
    /// Abilities asking for objects (`spec::run::attack_can_do_something`).
    ///
    /// Off (the rulings): an attack whose Energy cost is paid may be declared even when its effect can't be done; it does
    /// nothing and the turn ends (APR A-01: an attack needs its Energy; RULES.md "Options that can't do anything" is about
    /// questions and the uses of Trainers and Abilities, not attacks).
    pub hide_attacks_that_can_do_nothing: bool,
    /// Every Prize card taken is chosen by its taker (the Prize question), even when which card can't matter: taking the
    /// last Prize cards, the Prize cards for a Knock Out that leaves the opponent no Pokémon in play, a player's Prize
    /// cards in the same Knock Outs as the opponent's game-winning ones, and an effect's "take a Prize card" with one left.
    ///
    /// Off (the rulings): a choice that can't change the game isn't asked (RULES.md "Options that can't do anything", user
    /// decision 2): taking every Prize card left (all of them are taken), the Prize cards once the opponent has no Pokémon
    /// in play (the game is decided: APR E, the winner is determined after every effect, `knockout::check_winner`) are
    /// taken without a question (the first ones), an effect's "take a Prize card" takes the only one left, and when a
    /// player takes their last Prize cards the other player's Prize cards of the same Knock Outs aren't taken (the game is
    /// decided). The winner is the same either way.
    pub prize_cards_always_chosen: bool,
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
    /// Whether a Trainer card can change the game is judged from the cards the player can see: a condition on which cards
    /// a hidden zone holds or could hold (the kinds of the cards of the deck or of the opponent's hand, the copies of a
    /// named card not yet seen, a card in the card pool that evolves from one of the player's Pokémon) isn't read, so the
    /// card is offered unless the known zones rule it out (e.g. Salvatore when no card in the pool evolves from the
    /// player's Pokémon: its search of the deck may find nothing; Love Ball when every copy of the opponent's Pokémon's
    /// names is known). Its printed conditions and every condition on a known zone (an empty discard pile, a full Bench,
    /// no Energy to discard, an empty deck) still keep it from being played (`spec::run::check_play`).
    ///
    /// Off (the rulings): every condition the rules' usability walk reads applies, those on what a deck could hold
    /// included (APR B-01 / D-02; id925, id956: "you cannot play a Trainer card for no net effect"; RULES.md "Options that
    /// can't do anything", user decision 2, batch 8 C10; user decision D10).
    pub trainer_conditions_on_hidden_cards_unread: bool,
    /// A copy attack ("choose 1 of ... attacks and use it as this attack": Night Joker, Foul Play, Metronome, Try to
    /// Imitate, Seek Inspiration, Gemstone Mimicry) chosen by another copy attack can be chosen, and does nothing: no
    /// choice of its own, nothing else of its text (e.g. Night Joker choosing another N's Zoroark ex's Night Joker ends
    /// there). A copy attack used otherwise (one of Mew ex's Memory Helix attacks) is unaffected; the attack it
    /// chooses is. `copy_attack::resume`.
    ///
    /// Off (the rulings): the copied copy attack is used like any other and makes its own choice (compendium id656; JP
    /// FAQ Slowking SCR 58), among the attacks not already being used in this attack (id657; RULES.md "A copied attack
    /// is the copycat's only while it is used", `Game::copy_chain`).
    pub copied_copy_attack_does_nothing: bool,
    /// Whether an Ability that attaches cards to a Pokémon ("attach ... from your hand to 1 of your Benched ... Pokémon")
    /// can be used is judged from the cards it would attach only: with such cards in the zone it attaches from it is
    /// offered even when none of the player's Pokémon may get them (e.g. Ethan's Ho-Oh ex's Golden Flame with Basic Fire
    /// Energy in hand and no Benched Ethan's Pokémon); used then, it attaches nothing (its question has no answer, so it
    /// isn't asked) and its once-per-turn use is spent. Printed conditions ("if you have ... in play") still apply.
    /// `spec::ops::cards::implied_ok` (the attach's precondition, read by the usability walk of an Ability).
    ///
    /// Off (the rulings): an Ability that would not change the game state can't be used (APR A-02 step 1: "you can't use
    /// the Ability if going through the effects of Step 2 below would not change the game state in any way"; id925,
    /// id2362; RULES.md "Using Trainers, Abilities and attacks"): an attach with no Pokémon to attach to does nothing.
    pub ability_attach_targets_unread: bool,
    /// An attack's choices are asked when the effect they belong to runs, after the damage: the attack has no step of
    /// choices before its damage (e.g. Team Rocket's Murkrow's Torment: the damage is done, then the attack to lock is
    /// chosen). Text the attack does "before doing
    /// damage" and the damage calculation are unchanged; every choice of the effects after the damage is made as the
    /// effect is carried out. `spec::run::run` (no `Phase::Choices`).
    ///
    /// Off (the rulings): every choice the attack's effects require is made before the damage and carried out after it
    /// (RULES.md "Attacks / Order", step D: the Rules Team's forum answer of 2026-04-18 on Tricky Steps vs
    /// Adrena-Pheromone: the Energy is chosen first, the damage and the flip happen, then the Energy is moved; compendium
    /// id220, id521, id2034: the choice is made, the damage calculated, then the choice carried out); a deck search and a
    /// choice under a coin or an "If" the damage can change are made after the damage, when their options exist.
    pub attack_choices_asked_when_their_effect_runs: bool,
}

impl TableRules {
    /// Every option's name, in the order of the fields.
    pub const NAMES: &'static [&'static str] = &[
        "ask_single_way_payments",
        "hide_attacks_that_can_do_nothing",
        "prize_cards_always_chosen",
        "looked_cards_shown_to_owner",
        "evolving_trainer_discarded_before_evolve_triggers",
        "trainer_conditions_on_hidden_cards_unread",
        "copied_copy_attack_does_nothing",
        "ability_attach_targets_unread",
        "attack_choices_asked_when_their_effect_runs",
    ];

    /// Set the option `name` (one of [`TableRules::NAMES`]); an unknown name is an error.
    pub fn set(&mut self, name: &str, on: bool) -> Result<(), String> {
        let field = match name {
            "ask_single_way_payments" => &mut self.ask_single_way_payments,
            "hide_attacks_that_can_do_nothing" => &mut self.hide_attacks_that_can_do_nothing,
            "prize_cards_always_chosen" => &mut self.prize_cards_always_chosen,
            "looked_cards_shown_to_owner" => &mut self.looked_cards_shown_to_owner,
            "evolving_trainer_discarded_before_evolve_triggers" => &mut self.evolving_trainer_discarded_before_evolve_triggers,
            "trainer_conditions_on_hidden_cards_unread" => &mut self.trainer_conditions_on_hidden_cards_unread,
            "copied_copy_attack_does_nothing" => &mut self.copied_copy_attack_does_nothing,
            "ability_attach_targets_unread" => &mut self.ability_attach_targets_unread,
            "attack_choices_asked_when_their_effect_runs" => &mut self.attack_choices_asked_when_their_effect_runs,
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

    /// Games with every table rule on play to their end with uniformly random answers (the benchmark's decks): every
    /// answer the pick mask allows is accepted, no game gets stuck, and the rules stay as set.
    #[test]
    fn games_with_every_table_rule_play_through() {
        let spec: serde_json::Value = serde_json::from_str(include_str!("../bench/spec.json")).unwrap();
        let decks: Vec<Vec<crate::carddb::DefId>> = spec["decks"].as_array().unwrap().iter().map(|d| crate::selfplay::expand_deck(&d["cards"]).unwrap()).collect();
        for seed in 0..96u32 {
            let (a, b) = (&decks[seed as usize % decks.len()], &decks[(seed as usize * 7 + 3) % decks.len()]);
            let mut g = crate::game::Game::new(seed);
            g.st.rules = TableRules::all_on();
            g.start([a, b]).unwrap();
            let mut rng = crate::rng::Rng::new(seed.wrapping_mul(2654435761));
            for _ in 0..6000 {
                let Some(sel) = g.select().unwrap() else { break };
                let mut picks: Vec<usize> = Vec::new();
                loop {
                    let (mask, stop) = g.pick_mask(&sel, &picks);
                    let allowed: Vec<usize> = (0..sel.options.len()).filter(|j| mask[*j] && (sel.allows_repeats() || !picks.contains(j))).collect();
                    let choices = allowed.len() + stop as usize;
                    if choices == 0 {
                        break;
                    }
                    let r = rng.index(choices);
                    if r == allowed.len() {
                        break;
                    }
                    picks.push(allowed[r]);
                    if sel.max_count <= 1 {
                        break;
                    }
                }
                if picks.is_empty() && !g.pick_mask(&sel, &picks).1 {
                    // No option can be picked and the empty list isn't a full answer: the prompt's null answer (as the
                    // benchmark's policy does).
                    let crate::game::Pending::Decision(pi) = g.pending() else { panic!("seed {seed}: no option at a turn decision") };
                    let pr = g.prompts.as_slice()[pi];
                    let res = [serde_json::Value::Null, serde_json::json!([])].iter().find_map(|w| g.decode_answer(&pr, w).ok());
                    let res = res.unwrap_or_else(|| panic!("seed {seed}: no answer to {:?}", g.describe_prompt(&pr)));
                    g.resolve(pi, res).unwrap();
                    continue;
                }
                g.answer(&sel, &picks).unwrap_or_else(|e| panic!("seed {seed}: answer {picks:?} refused: {e:?}"));
            }
            assert!(g.st.phase == crate::types::GamePhase::Finished, "seed {seed}: the game didn't end (turn {})", g.st.turn);
            assert_eq!(g.st.rules, TableRules::all_on());
        }
    }
}
