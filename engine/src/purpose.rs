//! Why a prompt is asked: the typed purpose every prompt declares (rules concepts, never cards).
//!
//! The engine reads only the purpose: `interface::context_for` maps it to the select context, `options` tests it for the
//! Confusion flip. The recorded descriptors' `message` string is a function of the purpose (`PromptPurpose::message`),
//! used only at the trace boundary (the prompt descriptor, the Python adapter, the probe tool).

macro_rules! purposes {
    ($($(#[$doc:meta])* $v:ident => $m:expr,)*) => {
        /// The rules purpose of a prompt.
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum PromptPurpose {
            $($(#[$doc])* $v,)*
        }

        impl PromptPurpose {
            /// Every purpose (for the unit tests).
            pub const ALL: &'static [PromptPurpose] = &[$(PromptPurpose::$v,)*];

            /// The message string of the recorded prompt descriptor (empty: the descriptor has no `message`). Trace
            /// boundary only: no engine rule reads it.
            pub const fn message(self) -> &'static str {
                match self {
                    $(PromptPurpose::$v => $m,)*
                }
            }
        }
    };
}

purposes! {
    // Prompts that are not a player's choice
    /// Waiting for the other player (no label).
    Wait => "",
    /// Shuffling a deck (no label).
    Shuffle => "",
    /// Cards shown to a player (a reveal, a look).
    Look => "CARDS_SHOWED_BY_THE_OPPONENT",
    /// A player had no Basic Pokémon (APR: Mulligan): shown to that player.
    MulliganYours => "SETUP_PLAYER_NO_BASIC",
    /// A player had no Basic Pokémon: shown to the other player.
    MulliganOpponents => "SETUP_OPPONENT_NO_BASIC",
    // Coin flips
    /// The flip that decides who goes first (setup, sudden death).
    FirstPlayerFlip => "SETUP_WHO_BEGINS_FLIP",
    /// The Confusion flip of an attack.
    ConfusionFlip => "FLIP_CONFUSION",
    /// The Burn flip between turns.
    BurnFlip => "FLIP_BURNED",
    /// The Sleep flip between turns.
    SleepFlip => "FLIP_ASLEEP",
    // Setup
    /// Choose the Active Pokémon (and the Bench) at setup.
    SetupActive => "CHOOSE_STARTING_POKEMONS",
    /// The winner of the flip chooses to go first or second.
    ChooseFirstPlayer => "GO_FIRST",
    /// How many cards to draw (the Mulligan draw).
    DrawCount => "WANT_TO_DRAW_CARDS",
    // "You may" questions
    /// "You may use ..." (an Ability, an effect, a re-flip).
    Activate => "WANT_TO_USE_ABILITY",
    /// "You may use the effect of an attack."
    MayUseAttackEffect => "WANT_TO_USE_EFFECT_OF_ATTACK",
    /// "You may draw cards until you have N cards in your hand."
    MayDrawUntilHandSize => "WANT_TO_DRAW_UNTIL_6",
    /// "You may discard a card."
    MayDiscardCard => "WANT_TO_DISCARD_CARD",
    /// "You may discard an Energy."
    MayDiscardEnergy => "WANT_TO_DISCARD_ENERGY",
    /// "You may switch."
    MaySwitch => "WANT_TO_SWITCH_POKEMON",
    /// "You may search your deck."
    MaySearchDeck => "SEARCH_DECK_FOR_CARD",
    // Choosing
    /// Choose a target (no label).
    ChooseTarget => "",
    /// Choose a Pokémon.
    ChoosePokemon => "CHOOSE_POKEMON",
    /// Choose a card.
    ChooseCard => "CHOOSE_CARD",
    /// Choose cards.
    ChooseCards => "CHOOSE_CARDS",
    /// Choose one of several options of an effect.
    ChooseOption => "CHOOSE_OPTION",
    /// Choose the order of simultaneous effects (APR E-03).
    OrderEffects => "CHOOSE_EFFECT_ORDER",
    /// Put cards in an order (APR E-04).
    OrderCards => "CHOOSE_CARDS_ORDER",
    /// Choose an attack to copy.
    CopyAttack => "CHOOSE_ATTACK_TO_COPY",
    /// Choose an attack that can't be used.
    DisableAttack => "CHOOSE_ATTACK_TO_DISABLE",
    /// Choose a card whose effect is used.
    CopyEffect => "CHOOSE_CARD_TO_COPY_EFFECT",
    // Switch, Active Spot
    /// The new Active Pokémon (after a Knock Out or a switch out).
    NewActive => "CHOOSE_NEW_ACTIVE_POKEMON",
    /// The Pokémon to switch with.
    SwitchPokemon => "CHOOSE_POKEMON_TO_SWITCH",
    // Prizes
    /// Take a Prize card.
    TakePrize => "CHOOSE_PRIZE_CARD",
    /// Discard a Prize card.
    DiscardPrize => "CHOOSE_PRIZE_CARD_TO_DISCARD",
    // Discard
    /// Discard a card.
    DiscardCard => "CHOOSE_CARD_TO_DISCARD",
    /// Discard Energy.
    DiscardEnergy => "CHOOSE_ENERGIES_TO_DISCARD",
    /// Discard Energy to pay the Retreat Cost.
    PayRetreatCost => "CHOOSE_ENERGY_TO_PAY_RETREAT_COST",
    /// Discard a Pokémon.
    DiscardPokemon => "CHOOSE_POKEMON_TO_DISCARD",
    /// Choose the Pokémon whose attached cards are discarded.
    DiscardAttachedFrom => "CHOOSE_POKEMON_TO_DISCARD_CARDS",
    /// Discard a Stadium, a Pokémon Tool or a Special Energy.
    DiscardStadiumToolOrSpecialEnergy => "DISCARD_STADIUM_OR_TOOL_OR_SPECIAL_ENERGY",
    // Zones
    /// Put a card into the hand.
    PutIntoHand => "CHOOSE_CARD_TO_HAND",
    /// Put a card onto the Bench.
    PutOntoBench => "CHOOSE_CARD_TO_PUT_ONTO_BENCH",
    /// Put a card into the deck.
    PutIntoDeck => "CHOOSE_CARD_TO_DECK",
    /// Put cards on the bottom of the deck.
    PutOnDeckBottom => "CHOOSE_CARDS_TO_PUT_ON_BOTTOM_OF_THE_DECK",
    /// Shuffle a Pokémon (and its attached cards) into the deck.
    ShufflePokemonIntoDeck => "CHOOSE_POKEMON_TO_SHUFFLE",
    /// Put a Pokémon and its attached cards into the hand.
    PickUpPokemon => "CHOOSE_POKEMON_TO_PICK_UP",
    // Evolve
    /// The Evolution card to evolve with.
    EvolutionCard => "CHOOSE_CARD_TO_EVOLVE",
    /// The Pokémon to evolve.
    EvolvePokemon => "CHOOSE_POKEMON_TO_EVOLVE",
    // Attach
    /// The card to attach.
    AttachCard => "CHOOSE_CARD_TO_ATTACH",
    /// The Pokémon cards are attached to.
    AttachTo => "CHOOSE_POKEMON_TO_ATTACH_CARDS",
    /// Attach Energy cards (from a zone).
    AttachEnergy => "ATTACH_ENERGY_CARDS",
    /// Attach Energy to Benched Pokémon.
    AttachEnergyToBench => "ATTACH_ENERGY_TO_BENCH",
    /// Move Energy from one Pokémon to another.
    MoveEnergy => "MOVE_ENERGY_CARDS",
    // Damage
    /// Move damage counters.
    MoveDamage => "MOVE_DAMAGE",
    /// Choose the Pokémon that is damaged (or gets damage counters).
    DamageTarget => "CHOOSE_POKEMON_TO_DAMAGE",
    /// Choose the Pokémon that is healed.
    HealTarget => "CHOOSE_POKEMON_TO_HEAL",
}

#[cfg(test)]
mod tests {
    use super::PromptPurpose;
    use std::collections::HashSet;

    /// The distinct non-empty prompt messages of the golden traces (corpus survey, events batch 9 C10): each is the
    /// message of exactly one purpose, so the recorded descriptors are unchanged.
    const GOLDEN_MESSAGES: &[&str] = &[
        "CHOOSE_CARD_TO_HAND", "CHOOSE_CARD_TO_DECK", "CHOOSE_CARD_TO_DISCARD", "CHOOSE_POKEMON_TO_SWITCH", "CHOOSE_PRIZE_CARD",
        "CHOOSE_NEW_ACTIVE_POKEMON", "ATTACH_ENERGY_CARDS", "CHOOSE_STARTING_POKEMONS", "CHOOSE_CARD_TO_PUT_ONTO_BENCH",
        "CHOOSE_ENERGY_TO_PAY_RETREAT_COST", "GO_FIRST", "CHOOSE_CARD", "WANT_TO_USE_ABILITY", "CHOOSE_CARD_TO_EVOLVE",
        "CHOOSE_POKEMON_TO_HEAL", "CHOOSE_POKEMON_TO_DAMAGE", "CHOOSE_CARD_TO_ATTACH", "CHOOSE_CARDS_ORDER", "WANT_TO_DRAW_CARDS",
        "CHOOSE_POKEMON_TO_EVOLVE", "MOVE_ENERGY_CARDS", "CHOOSE_POKEMON_TO_DISCARD_CARDS", "CHOOSE_CARDS", "CHOOSE_CARD_TO_COPY_EFFECT",
        "MOVE_DAMAGE", "CHOOSE_CARDS_TO_PUT_ON_BOTTOM_OF_THE_DECK", "CHOOSE_OPTION", "CHOOSE_POKEMON", "CHOOSE_POKEMON_TO_PICK_UP",
        "ATTACH_ENERGY_TO_BENCH", "CHOOSE_POKEMON_TO_ATTACH_CARDS", "CHOOSE_ATTACK_TO_COPY", "CHOOSE_POKEMON_TO_DISCARD",
        "CHOOSE_ENERGIES_TO_DISCARD", "CHOOSE_ATTACK_TO_DISABLE", "WANT_TO_DRAW_UNTIL_6", "WANT_TO_DISCARD_ENERGY",
        "DISCARD_STADIUM_OR_TOOL_OR_SPECIAL_ENERGY",
    ];

    #[test]
    fn every_golden_message_is_one_purposes() {
        for m in GOLDEN_MESSAGES {
            let n = PromptPurpose::ALL.iter().filter(|p| p.message() == *m).count();
            assert_eq!(n, 1, "{m}");
        }
    }

    #[test]
    fn a_message_belongs_to_one_purpose() {
        let mut seen = HashSet::new();
        for p in PromptPurpose::ALL {
            let m = p.message();
            assert!(m.is_empty() || seen.insert(m), "{m} is the message of two purposes");
        }
        // The labelless purposes: no `message` in the descriptor.
        for p in [PromptPurpose::Wait, PromptPurpose::Shuffle, PromptPurpose::ChooseTarget] {
            assert_eq!(p.message(), "");
        }
    }

    fn rs_files(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        for e in std::fs::read_dir(dir).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                rs_files(&p, out);
            } else if p.extension().is_some_and(|x| x == "rs") {
                out.push(p);
            }
        }
    }

    /// No engine code reads a prompt message: no Twinleaf message literal outside this file, and `.message()` only at the
    /// trace boundary (the prompt descriptor, the adapter's select, the probe and audit tools).
    #[test]
    fn no_message_string_is_read_by_the_engine() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut files = Vec::new();
        rs_files(&root.join("src"), &mut files);
        rs_files(&root.join("examples"), &mut files);
        let boundary = ["src/prompts.rs", "src/interface.rs", "src/bin/probe.rs", "examples/audit_deck.rs"];
        let literals = [["\"CHOOSE", "_"].concat(), ["\"WANT", "_"].concat(), ["\"GO", "_FIRST\""].concat()];
        let call = [".mess", "age()"].concat();
        for f in files {
            let rel = f.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/");
            if rel == "src/purpose.rs" || rel.starts_with("src/gen/") {
                continue;
            }
            let s = std::fs::read_to_string(&f).unwrap();
            for l in &literals {
                assert!(!s.contains(l.as_str()), "{rel}: a prompt message literal");
            }
            if s.contains(call.as_str()) {
                assert!(boundary.contains(&rel.as_str()), "{rel}: reads a prompt message outside the trace boundary");
            }
        }
    }

    /// The context the old string mapping gave a prompt message (exact strings, then `contains`).
    fn legacy_context(message: &str) -> crate::interface::SelectContext {
        match message {
            "CHOOSE_STARTING_POKEMONS" => crate::interface::SelectContext::SetupActivePokemon,
            "CHOOSE_NEW_ACTIVE_POKEMON" | "CHOOSE_POKEMON_TO_SWITCH" => crate::interface::SelectContext::Switch,
            "CHOOSE_PRIZE_CARD" => crate::interface::SelectContext::ToHand,
            "CHOOSE_PRIZE_CARD_TO_DISCARD" => crate::interface::SelectContext::Discard,
            "CHOOSE_ENERGY_TO_PAY_RETREAT_COST" | "CHOOSE_ENERGIES_TO_DISCARD" => crate::interface::SelectContext::DiscardEnergy,
            "WANT_TO_DRAW_CARDS" => crate::interface::SelectContext::DrawCount,
            "GO_FIRST" => crate::interface::SelectContext::IsFirst,
            "WANT_TO_USE_ABILITY" => crate::interface::SelectContext::Activate,
            m if m.contains("DISCARD") => crate::interface::SelectContext::Discard,
            m if m.contains("HAND") => crate::interface::SelectContext::ToHand,
            m if m.contains("BENCH") => crate::interface::SelectContext::ToBench,
            m if m.contains("DECK") => crate::interface::SelectContext::ToDeck,
            _ => crate::interface::SelectContext::EffectTarget,
        }
    }

    /// Every purpose maps to a context, equal to the old string mapping for every purpose a player answers. The
    /// purposes that are not a choice (waits, shuffles, flips, reveals, the Mulligan notices) had the catch-all context
    /// and never reach a decision select; they now name their own.
    #[test]
    fn context_equals_the_old_string_mapping() {
        use PromptPurpose as P;
        let not_a_choice = [P::Wait, P::Shuffle, P::Look, P::MulliganYours, P::MulliganOpponents, P::FirstPlayerFlip, P::ConfusionFlip, P::BurnFlip, P::SleepFlip];
        for p in PromptPurpose::ALL {
            if not_a_choice.contains(p) {
                continue;
            }
            assert_eq!(crate::interface::context_for(*p) as u8, legacy_context(p.message()) as u8, "{:?}", p);
        }
    }
}
