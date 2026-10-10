//! Backtrack Badge (PBL; Twinleaf "Retry Badge M5", class BacktrackBadge,
//! tool): once during your turn, after you flip any coins for an attack of
//! the [C] Pokémon this card is attached to, you may ignore the results of
//! those coin flips and begin flipping those coins again.
//!
//! A re-flip declaration (events batch 7, user decision D7; `Modifier::Reflip`, offered by
//! `engine::condition::flip_coin` / `flip_sequence` before the flips' results are used): the group of flips of an effect
//! caused by an attack of the Pokémon holding the Badge, a [C] Pokémon (its printed type: the attacking Pokémon itself,
//! not the Pokémon whose attack it copies; JP FAQ Slowking's Seek Inspiration choosing Stoutland's attack). All the
//! attack's flips are flipped again from the first (id2416; JP FAQ Mega Kangaskhan ex / Mega Audino ex); a Confusion
//! flip isn't an attack's flip for an effect (id2417), nor an Ability's flips (JP FAQ Cinccino ex). The re-flip's flips
//! are new CoinFlip events with the attack's cause, not the Tool's. Once during your turn: the marker
//! COIN_REFLIP_AGAIN_USED on its player, cleared at the end of their turn.
use crate::cause::CauseKind;
use crate::markers::COIN_REFLIP_AGAIN_USED;
use crate::spec::event::CoinPurpose;
use crate::spec::prelude::*;
use crate::types::ct;

pub static SPEC: CardSpec = CardSpec {
    class: "BacktrackBadge",
    passives: &[Passive {
        origin: RuleSource::Tool,
        modifier: Modifier::Reflip(ReflipSpec {
            on: EventPred::All(&[
                EventPred::Kind(EventKind::CoinFlip),
                EventPred::Purpose(CoinPurpose::Effect),
                EventPred::Cause(CausePred::All(&[CausePred::Kind(CauseKind::Attack), CausePred::Holder, CausePred::Card(Pred::PokemonType(ct::COLORLESS))])),
            ]),
            once: COIN_REFLIP_AGAIN_USED,
        }),
    }],
    triggers: &[
        // The once-per-turn use ends with the turn.
        Trigger {
            origin: RuleSource::Tool,
            event: Event::OnEndTurn(OnEndTurnSpec { whose: Turn::Owner }),
            steps: &[Step::new(Op::ClearMarker(ClearMarkerSpec { scope: MarkerScope::Player(Who::Me), name: "COIN_REFLIP_AGAIN_USED", from: MarkerFrom::Any }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
