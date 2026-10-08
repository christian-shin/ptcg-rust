//! Blowtorch (PFL): discard a Basic [R] Energy card from your hand to use
//! this card. Discard a Pokémon Tool or Special Energy card from 1 of your
//! opponent's Pokémon, or discard a Stadium in play.
//!
//! Twinleaf: the energy choice (min 1, name "Fire Energy") is queued without
//! waiting; the SelectPrompt offers Tool / Special Energy / Stadium from the
//! counts taken when the card was played (the Special Energy blocked list is
//! taken after the energy discard). After the chosen action queues its
//! prompt (or discards the Stadium), the card moves supporter→discard.
//! The multi-tool branch chooses from `cardList.cards`, where Tools never
//! are.
use crate::spec::prelude::*;

const HAND: ZoneRef = ZoneRef(Who::Me, Zone::Hand);
const FIRE_ENERGY: Pred = Pred::All(&[Pred::BasicEnergy, Pred::Name("Fire Energy")]);
const OPP_POKEMON: SlotSel = SlotSel::Pokemon(Who::Opp);
const MY_STADIUM: ZoneRef = ZoneRef(Who::Me, Zone::Stadium);
const OPP_STADIUM: ZoneRef = ZoneRef(Who::Opp, Zone::Stadium);
const HAS_TOOL: Cond = Cond::AnySlot(OPP_POKEMON, SlotPred::HasTool);
const HAS_SPECIAL: Cond = Cond::AnySlot(OPP_POKEMON, SlotPred::HasSpecialEnergy);
const STADIUM_IN_PLAY: Cond = Cond::Any(&[Cond::Nonempty(MY_STADIUM, Pred::Any), Cond::Nonempty(OPP_STADIUM, Pred::Any)]);
const ATTACHED: ZoneRef = ZoneRef(Who::Opp, Zone::Attached(SlotExpr::Picked));

pub static SPEC: CardSpec = CardSpec {
    class: "Blowtorch",
    // Discard a Basic [R] Energy card from your hand to use this card. Discard a Pokémon Tool or
    // Special Energy card from 1 of your opponent's Pokémon, or discard a Stadium in play.
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[Cond::Nonempty(HAND, FIRE_ENERGY), Cond::Any(&[HAS_TOOL, HAS_SPECIAL, STADIUM_IN_PLAY])],
        steps: &[
            Step::new(Op::Pick(PickSpec { from: HAND, predicate: FIRE_ENERGY, bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, into: 0, msg: "CHOOSE_CARD_TO_DISCARD", ..PickSpec::DEFAULT })),
            Step::new(Op::Move(MoveSpec { from: HAND, to: ZoneRef(Who::Me, Zone::Discard), cards: CardSel::Chosen(0), ..MoveSpec::DEFAULT })),
            Step::new(Op::Choose(ChooseSpec {
                chooser: Who::Me,
                msg: "DISCARD_STADIUM_OR_TOOL_OR_SPECIAL_ENERGY",
                options: &[
                    ChoiceBranch {
                        label: "CHOICE_TOOL",
                        avail: HAS_TOOL,
                        body: &[
                            Step::new(Op::PickSlot(PickSlotSpec {
                                chooser: Who::Me,
                                among: SlotSel::Filtered(&OPP_POKEMON, SlotPred::HasTool),
                                msg: "CHOOSE_POKEMON_TO_DISCARD_CARDS",
                            })),
                            Step::new(Op::Move(MoveSpec {
                                from: ATTACHED,
                                to: ZoneRef(Who::Opp, Zone::Discard),
                                cards: CardSel::Tools(SlotExpr::Picked),
                                ..MoveSpec::DEFAULT
                            })),
                        ],
                    },
                    ChoiceBranch {
                        label: "CHOICE_SPECIAL_ENERGY",
                        avail: HAS_SPECIAL,
                        body: &[
                            Step::new(Op::PickSlot(PickSlotSpec {
                                chooser: Who::Me,
                                among: SlotSel::Filtered(&OPP_POKEMON, SlotPred::HasSpecialEnergy),
                                msg: "CHOOSE_POKEMON_TO_DISCARD_CARDS",
                            })),
                            Step::new(Op::Pick(PickSpec {
                                from: ATTACHED,
                                predicate: Pred::SpecialEnergy,
                                bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
                                into: 0,
                                msg: "CHOOSE_CARD_TO_DISCARD",
                                ..PickSpec::DEFAULT
                            })),
                            Step::new(Op::Move(MoveSpec { from: ATTACHED, to: ZoneRef(Who::Opp, Zone::Discard), cards: CardSel::Chosen(0), ..MoveSpec::DEFAULT })),
                        ],
                    },
                    ChoiceBranch {
                        label: "CHOICE_STADIUM",
                        avail: STADIUM_IN_PLAY,
                        body: &[
                            Step::new(Op::If(IfSpec {
                                cond: Cond::Nonempty(MY_STADIUM, Pred::Any),
                                yes: &[Step::new(Op::Move(MoveSpec { from: MY_STADIUM, to: ZoneRef(Who::Me, Zone::Discard), cards: CardSel::All, ..MoveSpec::DEFAULT }))],
                                no: &[],
                            })),
                            Step::new(Op::If(IfSpec {
                                cond: Cond::Nonempty(OPP_STADIUM, Pred::Any),
                                yes: &[Step::new(Op::Move(MoveSpec { from: OPP_STADIUM, to: ZoneRef(Who::Opp, Zone::Discard), cards: CardSel::All, ..MoveSpec::DEFAULT }))],
                                no: &[],
                            })),
                        ],
                    },
                ],
            })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
