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
    // Setup and the turn
    /// Choose the Active Pokémon (and the Bench) at setup.
    SetupActive => "CHOOSE_STARTING_POKEMONS",
    /// The winner of the flip chooses to go first or second.
    ChooseFirstPlayer => "GO_FIRST",
    /// How many cards to draw (the Mulligan draw).
    DrawCount => "WANT_TO_DRAW_CARDS",
    // Questions and options
    /// A "you may" question (an Ability, an effect, a cost, a re-flip): yes or no.
    Activate => "WANT_TO_USE_ABILITY",
    /// One of several labeled branches of an effect (the only choice with no outcome of its own: each branch has its own).
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
    /// Choose the Pokémon a lasting effect applies to (the effect itself is another step).
    EffectTarget => "CHOOSE_POKEMON",
    // The Active Spot
    /// The new Active Pokémon after a Knock Out (APR: promote a Benched Pokémon).
    NewActive => "CHOOSE_NEW_ACTIVE_POKEMON",
    /// The Benched Pokémon an effect switches into the Active Spot.
    SwitchPokemon => "CHOOSE_POKEMON_TO_SWITCH",
    // Prizes
    /// Take a Prize card.
    TakePrize => "CHOOSE_PRIZE_CARD",
    /// Turn a Prize card face up.
    TurnPrizeFaceUp => "CHOOSE_PRIZE_CARD_TO_TURN_FACE_UP",
    /// Choose a Prize card to look at (it stays face down).
    LookAtPrize => "CHOOSE_PRIZE_CARD_TO_LOOK_AT",
    // Discard
    /// Discard a card (from a hand, a Pokémon's attached cards, a Bench slot).
    DiscardCard => "CHOOSE_CARD_TO_DISCARD",
    /// Discard Energy (a cost, a retreat, an effect).
    DiscardEnergy => "CHOOSE_ENERGIES_TO_DISCARD",
    // Zones
    /// Put a card (or a Pokémon with its attached cards) into the hand.
    PutIntoHand => "CHOOSE_CARD_TO_HAND",
    /// Put a card onto the Bench.
    PutOntoBench => "CHOOSE_CARD_TO_PUT_ONTO_BENCH",
    /// Put a card (or a Pokémon with its attached cards) into the deck.
    PutIntoDeck => "CHOOSE_CARD_TO_DECK",
    /// Put cards on the bottom of the deck.
    PutOnDeckBottom => "CHOOSE_CARDS_TO_PUT_ON_BOTTOM_OF_THE_DECK",
    // Evolution
    /// The Evolution card to evolve with.
    EvolutionCard => "CHOOSE_CARD_TO_EVOLVE",
    /// The Pokémon to evolve.
    EvolvePokemon => "CHOOSE_POKEMON_TO_EVOLVE",
    /// The Pokémon (or its card) that is devolved.
    Devolve => "CHOOSE_POKEMON_TO_DEVOLVE",
    /// The Pokémon card that is switched with a Pokémon in play, or the Pokémon whose card is replaced (the Swap event).
    SwapPokemon => "CHOOSE_POKEMON_TO_SWAP",
    // Attach
    /// The card to attach.
    AttachCard => "CHOOSE_CARD_TO_ATTACH",
    /// The Pokémon cards are attached to.
    AttachTo => "CHOOSE_POKEMON_TO_ATTACH_CARDS",
    // Moving Energy and damage counters
    /// Move Energy from one Pokémon to another.
    MoveEnergy => "MOVE_ENERGY_CARDS",
    /// Move damage counters from one Pokémon to another.
    MoveDamage => "MOVE_DAMAGE",
    // Damage, damage counters, healing
    /// The Pokémon that is damaged (or Knocked Out by an effect).
    DamageTarget => "CHOOSE_POKEMON_TO_DAMAGE",
    /// The Pokémon that gets damage counters.
    PutDamageCounters => "CHOOSE_POKEMON_TO_PUT_DAMAGE_COUNTERS",
    /// The Pokémon that is healed.
    HealTarget => "CHOOSE_POKEMON_TO_HEAL",
}

#[cfg(test)]
mod tests {
    use super::PromptPurpose;
    use std::collections::HashSet;

    #[test]
    fn a_message_belongs_to_one_purpose() {
        let mut seen = HashSet::new();
        for p in PromptPurpose::ALL {
            let m = p.message();
            assert!(m.is_empty() || seen.insert(m), "{m} is the message of two purposes");
        }
        // The labelless purposes (no `message` in the descriptor) are the two that are not a question at all.
        let labelless: Vec<_> = PromptPurpose::ALL.iter().filter(|p| p.message().is_empty()).collect();
        assert_eq!(labelless, [&PromptPurpose::Wait, &PromptPurpose::Shuffle]);
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

    /// Every purpose names one select context, the one of the outcome it asks for (the selection interface's contexts): the table
    /// lists every purpose (the test fails when a variant is missing from it) and the mapping is checked against it.
    #[test]
    fn every_purpose_names_its_context() {
        use crate::interface::SelectContext as C;
        use PromptPurpose as P;
        let table: &[(P, C)] = &[
            (P::Wait, C::EffectTarget),
            (P::Shuffle, C::Shuffle),
            (P::Look, C::Look),
            (P::MulliganYours, C::Mulligan),
            (P::MulliganOpponents, C::Mulligan),
            (P::FirstPlayerFlip, C::CoinHead),
            (P::ConfusionFlip, C::CoinHead),
            (P::BurnFlip, C::CoinHead),
            (P::SleepFlip, C::CoinHead),
            (P::SetupActive, C::SetupActivePokemon),
            (P::ChooseFirstPlayer, C::IsFirst),
            (P::DrawCount, C::DrawCount),
            (P::Activate, C::Activate),
            (P::ChooseOption, C::EffectTarget),
            (P::OrderEffects, C::SkillOrder),
            (P::OrderCards, C::ToDeck),
            (P::CopyAttack, C::Attack),
            (P::DisableAttack, C::DisableAttack),
            (P::CopyEffect, C::EffectTarget),
            (P::EffectTarget, C::EffectTarget),
            (P::NewActive, C::ToActive),
            (P::SwitchPokemon, C::Switch),
            (P::TakePrize, C::ToHand),
            (P::TurnPrizeFaceUp, C::Look),
            (P::LookAtPrize, C::Look),
            (P::DiscardCard, C::Discard),
            (P::DiscardEnergy, C::DiscardEnergy),
            (P::PutIntoHand, C::ToHand),
            (P::PutOntoBench, C::ToBench),
            (P::PutIntoDeck, C::ToDeck),
            (P::PutOnDeckBottom, C::ToDeckBottom),
            (P::EvolutionCard, C::EvolvesTo),
            (P::EvolvePokemon, C::EvolvesFrom),
            (P::Devolve, C::Devolve),
            (P::SwapPokemon, C::EffectTarget),
            (P::AttachCard, C::AttachTo),
            (P::AttachTo, C::AttachFrom),
            (P::MoveEnergy, C::SwitchEnergyCard),
            (P::MoveDamage, C::DamageCounter),
            (P::DamageTarget, C::Damage),
            (P::PutDamageCounters, C::DamageCounter),
            (P::HealTarget, C::Heal),
        ];
        assert_eq!(table.len(), PromptPurpose::ALL.len(), "the table lists every purpose once");
        for p in PromptPurpose::ALL {
            let (_, want) = table.iter().find(|(q, _)| q == p).unwrap_or_else(|| panic!("{p:?} is not in the table"));
            assert_eq!(crate::interface::context_for(*p) as u8, *want as u8, "{p:?}");
        }
    }
}
