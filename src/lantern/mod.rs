use std::{collections::HashMap, ops::{Deref, DerefMut}};

use anyhow::{Result, bail};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct DefKey(String);

/// Lantern Intermediate Representation format.
/// This is the primary structure used to manipulate
/// mod content.
#[derive(Serialize, Deserialize, Debug)]
pub struct Lantern {
    aspects: HashMap<DefKey, Aspect>,
    cards:   HashMap<DefKey, Card>,
    decks:  HashMap<DefKey, Deck>,
    recipes: HashMap<DefKey, Recipe>,
    verbs: HashMap<DefKey, Verb>,
    legacies: HashMap<DefKey, Legacy>,
    endings: HashMap<DefKey, Ending>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Aspect {
    id: DefKey,
    label: String,
    description: String,
    icon: String,
    verbicon: String,
    induces: (DefKey, Probability),
    #[deprecated(since="0.0.0", note="Aspect Decay API is not yet stable")]
    decay: DefKey,
    hidden: bool,

    no_art_needed: bool,
    #[deprecated(since="0.0.0", note="XTriggers API is not yet stable")]
    xtriggers: (),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Card {
    id: DefKey,
    label: String,
    description: String,
    icon: String,
    verbicon: String,
    induces: (DefKey, Probability),
    #[deprecated(since="0.0.0", note="Aspect Decay API is not yet stable")]
    decay: DefKey,
    hidden: bool,

    aspects: HashMap<DefKey, usize>,
    lifetime: usize,
    resaturate: bool,
    unique: bool,
    uniqueness_group: DefKey,

    slots: HashMap<DefKey, Vec<Slot>>,

    #[deprecated(since="0.0.0", note="XTriggers API is not yet stable")]
    xtriggers: (),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Deck {
    id: DefKey,
    label: String,
    description: String,
    reset_on_exhaustion: bool,

    default: DefKey,
    cards: Vec<DefKey>,
    draw_messages: HashMap<DefKey, String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum EndingFlavorKind {
    Grand,
    Mellancholy,
    Pale,
    Vile,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum MutationOperation {
    Set(usize),
    Add(isize),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ChallengeType {
    Base,
    Advanced,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AltRecipeLink {
    id: DefKey,
    additional: bool,
    chance: Probability,
    challenges: HashMap<DefKey, ChallengeType>,
    expulsions: HashMap<DefKey, usize>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Recipe {
    id: DefKey,
    verb: DefKey,
    label: String,
    description: String,
    end_description: String,
    burn: Option<String>,
    portal: Option<String>,

    requirements: Vec<DefKey>,
    table_requirements: Vec<DefKey>,
    extant_requirements: Vec<DefKey>,

    max_executions: usize,
    warmup: usize,
    craftable: bool,
    hint_only: bool,
    //TODO this might need to be an enum
    important: bool,
    slot: Option<Slot>,

    effects: HashMap<DefKey, isize>,
    purge: HashMap<DefKey, usize>,
    aspects: HashMap<DefKey, isize>,
    draws: HashMap<DefKey, isize>,
    // HashMap<Target, HashMap<Aspect, MutationOperation>>
    mutations: HashMap<DefKey, HashMap<DefKey, MutationOperation>>,

    halt: Option<DefKey>,
    delete: Option<DefKey>,

    ending: Option<DefKey>,
    ending_flavor: Option<EndingFlavorKind>,

    alt: Vec<AltRecipeLink>,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Slot {
    id: DefKey,
    label: String,
    description: String,
    consumes: bool,
    greedy: bool,
    // TODO these might need to be HashMaps? Spec is unclear.
    requirements: Vec<DefKey>,
    forbidden: Vec<DefKey>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Verb {
    id: DefKey,
    label: String,
    description: String,
    slot: Slot,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Legacy {
    id: DefKey,
    label: String,
    description: String,
    start_description: String,
    image: String,

    starting_verb: DefKey,
    starting_cards: HashMap<DefKey, usize>,
    status_bar_elems: Vec<DefKey>,

    exclude_after_legacies: Vec<DefKey>,
    new_start: bool,
    from_ending: DefKey,
    available_without_ending_match: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum EndingMusicKind {
    Grand,
    Melancholy,
    Vile,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum EndingAnimationKind {
    DramaticLight,
    DramaticLightCool,
    DramaticLightEvil,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Ending {
    id: DefKey,
    label: String,
    description: String,
    image: String,
    music: Option<EndingMusicKind>,
    animation: Option<EndingAnimationKind>,
}

/// u8 clamped from 0-100
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct Probability {
    inner: u8
}

impl Probability {
    pub fn new(value: u8) -> Result<Self> {
        match value {
            0..=100 => Ok(Self{ inner: value }),
            _ => bail!("a probability value must be an integer in the range 0..=100"),
        }
    }
}

impl TryFrom<u8> for Probability {
    type Error = anyhow::Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<Probability> for u8 {
    fn from(p: Probability) -> Self {
        p.inner
    }
}

impl Deref for Probability {
    type Target = u8;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Probability {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

