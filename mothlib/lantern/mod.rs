use std::{collections::HashMap, ops::{Deref, DerefMut}, fmt::Display, borrow::Borrow};
use anyhow::{Result, bail};
use either::Either;
use serde::{Serialize, Deserialize};

pub mod json;

/// An ID referencing an in-game component.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct DefKey(pub String);
impl Display for DefKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Lantern Intermediate Representation format.
/// This is the primary structure used to manipulate
/// mod content.
/// 
/// The Lantern struct represents an entire
/// mod. 
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Lantern {
    /// List of all attributes that apply to the whole mod.
    attributes: Vec<Attribute>,
    /// List of metadata for each namespace.
    /// The keys are the full path to the namespace.
    namespaces: HashMap<DefKey, NamespaceMeta>,

    aspects: HashMap<DefKey, Aspect>,
    cards:   HashMap<DefKey, Card>,
    decks:  HashMap<DefKey, Deck>,
    recipes: HashMap<DefKey, Recipe>,
    verbs: HashMap<DefKey, Verb>,
    legacies: HashMap<DefKey, Legacy>,
    endings: HashMap<DefKey, Ending>,
}

/// An Attribute is one or more defkeys, 
/// it is handled by extensions that read them.
/// Apart from a few builtins, Crucible
/// does not interact with attributes.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Attribute{ pub key: DefKey, pub value: Option<json::Value> }

/// A namespace is a collection of components
/// which describe its position within the
/// namespace hierarchy, and a collection
/// of attributes that have been applied to it.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NamespaceMeta {
    components: Vec<DefKey>,
    attributes: Vec<Attribute>,
}

/// Aspects are one of the two variants of the type
/// engine calls "Elements". Aspects are more
/// limited than cards, and act as metadata
/// that can be applied to other engine components.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Aspect {
    /// This is the in-game representation for
    /// this aspect. It is made up of the name
    /// of the aspect, e.g. "lantern", and the
    /// namespace it is defined in, e.g.
    /// "core.aspects" -> id = "core.aspects.lantern"
    pub id: DefKey,
    /// This is the title text that appears in the
    /// dialogue created when clicking on the aspect.
    pub label: String,
    /// This is the body text that appears in the
    /// dialogue created when clicking on the aspect.
    pub description: String,
    /// If defined, the engine will set the icon
    /// of this aspect to the image with this name 
    /// (sans extension) found in either the game's 
    /// base assets or `<MOD_ROOT>/images/<TYPE>`.
    /// If None, the engine will search the same
    /// location for an image with the same name
    /// as the full id of the aspect.
    pub icon: Option<String>,
    /// If defined, the engine will set the icon
    /// of this aspect when it is emitted as a verb
    /// to the image with this name (sans extension) 
    /// found in either the game's base assets or 
    /// `<MOD_ROOT>/images/<TYPE>`.
    /// If None, the engine will use the default
    /// aspect icon.
    pub verbicon: Option<String>,
    /// Whenever this aspect is present and face-up 
    /// (ie: not from effects or deckeffects) at the 
    /// conclusion of a recipe chain, the induced recipe 
    /// may be created as a new situation token depending 
    /// on the designated chance. This will not occur 
    /// if an active token with the actionId of the 
    /// specified Verb is already on the table, but 
    /// it will ignore any requirements, extantreqs, 
    /// or tablereqs associated with the induced recipe.
    pub induces: Option<(DefKey, Probability)>,
    /// Specifies the card that other cards with this
    /// element will decay to when they are purged.
    pub decays_to: Option<DefKey>,
    /// If true, this aspect will not be shown in any in-game aspect lists.
    /// Implies "noArtNeeded" in the Cultist Simulator
    /// core schema.
    pub hidden: bool,
    /// The list of [Xtrigger]s to run on this aspect when their
    /// conditions are met.
    pub xtriggers: Vec<Xtrigger>,
    /// Any other JSON members not otherwise specified in this struct.
    pub others: HashMap<DefKey, json::Value>,
}

/// Cards are one of the two varians of the type
/// the engine calls "Elements". Cards are nouns,
/// things that can be acted upon by the player
/// or the engine using a Verb.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Card {
    /// This is the in-game representation for
    /// this card. It is made up of the name
    /// of the card, e.g. "fragmentlantern", and the
    /// namespace it is defined in, e.g.
    /// "core.cards" -> id = "core.cards.fragmentlantern"
    pub id: DefKey,
    /// This is the title text that appears on the
    /// card when it is on the table as well as the
    /// title of the dialogue created when clicking 
    /// on the card.
    pub label: String,
    /// This is the body text that appears in the
    /// dialogue created when clicking on the card.
    pub description: String,
    /// If defined, the engine will set the icon
    /// of this card to the image with this name 
    /// (sans extension) found in either the game's 
    /// base assets or `<MOD_ROOT>/images/<TYPE>`.
    /// If None, the engine will search the same
    /// location for an image with the same name
    /// as the full id of the aspect.
    pub icon: Option<String>,
    /// If defined, the engine will set the icon
    /// of this card when it is emitted as a verb
    /// to the image with this name (sans extension) 
    /// found in either the game's base assets or 
    /// `<MOD_ROOT>/images/<TYPE>`.
    /// If None, the engine will use the default
    /// aspect icon.
    pub verbicon: Option<String>,
    /// Whenever this card is present and face-up 
    /// (ie: not from effects or deckeffects) at the 
    /// conclusion of a recipe chain, the induced recipe 
    /// may be created as a new situation token depending 
    /// on the designated chance. This will not occur 
    /// if an active token with the actionId of the 
    /// specified Verb is already on the table, but 
    /// it will ignore any requirements, extantreqs, 
    /// or tablereqs associated with the induced recipe.
    pub induces: Option<(DefKey, Probability)>,
    /// Specifies the card that this card
    /// will decay to when they are purged.
    pub decays_to: Option<DefKey>,
    /// If true, this card will not be shown in any in-game aspect lists.
    /// Implies "noArtNeeded" in the Cultist Simulator
    /// core schema.
    pub hidden: bool,
    /// The list of aspects this card has by default.
    /// Each aspect on the card also has an associated degree,
    /// which is how much of that aspect is on the card.
    pub aspects: HashMap<DefKey, u32>,
    /// This is how long the card can last on the table before it decays.
    /// If None or 0, the card will not decay.
    pub lifetime: Option<u32>,
    /// Only applies to elements with lifetimes. 
    /// Normally, as the lifetime reaches 0, the art will 
    /// desaturate. If this value is true, the card will 
    /// start desaturated and gain saturation as it reaches 
    /// 0 (like depleted ability cards, such as Health [Fatigued]).
    pub resaturate: bool,
    /// If true, only one of this card can be present on the board.
    /// Spawning a new one will cause the old one to fade.
    pub unique: bool,
    /// Any other elements with the same aspect for their Uniqueness Group 
    /// cannot exist simultaneously on the board. 
    /// Placing one onto the board will cause all others on the board to vanish, 
    /// just like with the “unique” property.
    /// 
    /// The uniqueness group is technically an aspect like any other, 
    /// and can be used for requirements and refinements. 
    ///
    /// This aspect **must** exist! If a string is used instead of an 
    /// aspect ID, the uniqueness group will work as intended, but 
    /// any base recipes that contain any element with this undeclared 
    /// uniqueness group will be unable to spawn additional recipes.
    pub uniqueness_group: Option<DefKey>,
    /// This field maps sets of slots to the verbs they appear in
    /// when this card is inserted into that verb.
    pub slots: HashMap<DefKey, Vec<Slot>>,
    /// The list of [Xtrigger]s to run on this card when their
    /// conditions are met.
    pub xtriggers: Vec<Xtrigger>,
}


/// A deck is a collection of Cards which
/// recipes can randomly draw from to produce
/// cards.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Deck {
    /// This is the in-game representation for
    /// this deck. It is made up of the name
    /// of the deck, e.g. "mansuswoodresults", and the
    /// namespace it is defined in, e.g.
    /// "core.decks" -> id = "core.decks.mansuswoodresults"
    pub id: DefKey,
    /// This is the title text that appears on the
    /// dialog produced when a face-down card produced from it
    /// is clicked.
    pub label: String,
    /// This is the body text that appears appears on the
    /// dialog produced when a face-down card produced from it
    /// is clicked.
    pub description: String,
    /// If None, then the deck will reset itself
    /// once all its cards have been drawn.
    /// If Some, then the deck will supply a default
    /// card repeatedly once all other cards
    /// in the deck have been drawn. 
    pub default: Option<DefKey>,
    /// This is the list of cards that can be
    /// pulled from the deck.
    /// If this deck is a portal deck (i.e. it is 
    /// used by the "portals" function from either 
    /// the base game or a custom world made
    /// with Mothlib's Portals extension),
    /// then these messages will be shown
    /// when the corresponding card is drawn
    /// from that deck and the player returns
    /// to the table. If this deck is not a portal
    /// deck, these values are ignored.
    pub cards: Vec<(DefKey, Option<String>)>,
    pub is_portal_deck: bool,
}

/// Defines the types of colors a recipe's
/// progress circle can be set to.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum WarmupStyle {
    /// No style
    None,
    /// Gold
    Grand,
    /// Red
    Mellancholy,
    /// White
    Pale,
    /// Edge-green
    Vile,
    /// Changes the color of the recipe warmup timer to a yellow-gold color
    /// and plays a sound that lets you know something important is happening.
    Important,
}

/// Defines an operation that mutates
/// a record by some amount. It is also
/// used to define requirement comparison
/// operations (see [RecipeRequirement]).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ValueOperation {
    Set(u32),
    Add(i32),
}

/// Defines the type challenge used in 
/// a flow control conditional.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ChallengeKind {
    /// A base challenge scales the chance into 
    /// a histogram based on the level of the 
    /// specified aspect in the current recipe’s 
    /// aspect stack. The probabilities are 
    /// 0 for 0%, 1-4 for 30%, 5-9 for 70% and 10+ for 90%.
    Base,
    /// A base challenge scales the chance into 
    /// a histogram based on the level of the 
    /// specified aspect in the current recipe’s 
    /// aspect stack. The probabilities are 
    /// 0-5 for 0%, 6-10 for 10%, 11-15 for 30% 
    /// and 16-20 for 70%, and 20+ for 90%.
    Advanced,
}

/// Defines some element and magnitude
/// that must be present in some way
/// before a recipe can be crafted
/// or branched to.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum RecipeRequirement {
    /// Requires the element be present
    /// within the recipe’s element stack. 
    /// This can be used comparatively, 
    /// having a requirement of "element_1" : "element_2" 
    /// requires that you have a greater 
    /// than or equal quantity of "element_1" as "element_2". 
    /// Furthermore, negative numbers are 
    /// allowed, and function as "less than", 
    /// allowing -1 to represent 0, -2 to 
    /// represent having 1 or 0, etc. 
    Basic{ element: DefKey, amount: Either<ValueOperation, DefKey> },
    /// A Table requirement must be present on the game board, 
    /// and not in a Verb tile. Accordingly, elements that 
    /// are in slots, both for ongoing and yet unstarted recipes, 
    /// and elements that aren’t yet picked from the Verb 
    /// after the situation ended, are considered to be outside 
    /// of the tablereqs scope. Like [RecipeRequirement::Basic], Table
    /// Requirements support less than and comparisons.
    Table{ element: DefKey, amount: Either<ValueOperation, DefKey> },
    /// An Extant requirement must be present somewhere 
    /// within the game. They can be in another recipe, 
    /// in the player’s hand, or on the board. Like [RecipeRequirement::Basic], 
    /// Extant Requirements support less than and comparisons.
    Extant{ element: DefKey, amount: Either<ValueOperation, DefKey> },
}

/// Defines the conditions which, when satisfied,
/// will cause a recipe branch to be followed.
/// If `chance` is None and `requirements` is empty,
/// then this BranchCondition represents an
/// unconditional branch.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BranchCondition {
    /// A branch will only be followed if a random
    /// value between 1 and 100 is less than or equal
    /// to `chance`. If None, defaults to 100%.
    pub chance: Option<Probability>,
    /// A branch will only be followed if all its
    /// [RecipeRequirement]s are satisfied.
    /// If `requirements` is empty, then
    /// it will always be considered satisfied.
    pub requirements: Vec<RecipeRequirement>
}

/// Defines the branching behavior when
/// a branch with this item is followed.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SpawningKind {
    /// If a branch with this property is followed,
    /// a new situation token will be generated for
    /// the target recipe.
    Spawn,
    /// If a branch with this property is followed,
    /// the effects of [SpawningKind::Spawn] will
    /// be applied, and additionally the spawned
    /// verb will take from its parent recipe
    /// up to thespecified number of the source
    /// of the specified elements in the list.
    Expel(Vec<(DefKey, u32)>)
}

/// Defines the types of control flow instructions
/// supported by the scheme.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Branch {
    /// Defines a recipe to be followed once the previous recipe is finished.
    /// 
    /// It will only be followed if all previously defined Links do not
    /// have their requirements satisfied and its own requirements are satisfied.
    Link{ target: DefKey, condition: BranchCondition },
    /// Defines a recipe which will attempt to be followed every time the aspect
    /// stack of the enclosing recipe is changed. 
    /// 
    /// It will only be followed if 
    /// all previously defined Gotos do not have their requirements satisfied 
    /// and its own requirements are satisfied.
    /// 
    /// If it is successfully invoked, then the current recipe will
    /// be interrupted and replaced with the new recipe. When switching,
    /// the current timer will be kept but its effects, end_description, 
    /// and links will not execute. 
    Goto{ target: DefKey, condition: BranchCondition, action: Option<SpawningKind> }
}

/// 'HashMap<Target, HashMap<Aspect, MutationOperation>>'
/// 
/// Mutations are used to add or remove aspects from an element. 
/// Mutated aspects remain even when the element changes via 
/// xtrigger or decay.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Mutation {
    /// Used to find the card or aspect that will be modified.
    pub id: DefKey,
    /// The aspect we are adding/removing on the element. 
    pub aspect: DefKey,
    /// The operation to commit
    pub operation: ValueOperation,
}

/// A recipe is a "sentence". They
/// combine a Noun with a Verb in order to
/// produce some consequence in-game.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Recipe {
    /// This is the in-game representation for
    /// this deck. It is made up of the name
    /// of the deck, e.g. "exitmansuswood", and the
    /// namespace it is defined in, e.g.
    /// "core.recipes" -> id = "core.recipes.exitmansuswood"
    pub id: DefKey,
    /// The ID of the verb which this recipe executes in.
    pub verb: DefKey,
    /// The title of the verb dialogue will be set to this
    /// value when the recipe begins.
    pub label: String,
    /// The text body of the verb dialogue will be set to this
    /// value when the recipe begins, and will persist while
    /// the warmup runs.
    pub description: String,
    /// The text body of the verb dialogue will be set
    /// to this value when the recipe finishes successfully.
    /// This text is never displayed if the recipe routes to 
    /// another linked recipe, as the action is not considered 
    /// completed until a recipe ends without starting another one.
    pub end_description: String,
    /// This is the image filename of a png file 
    /// located in the “images/burns” folder that 
    /// you’d like to display on the board when 
    /// the recipe begins.
    pub burn: Option<String>,
    /// This is the property that tells a recipe to transition 
    /// into the Mansus and which door to open to. This causes 
    /// the recipe to draw a card from each deck associated 
    /// with the door and let you choose from them on the board.
    pub portal: Option<String>,
    /// Defines a set of [RecipeRequirement]s which must all
    /// be satisfied in order for the recipe to be executed.
    /// These requirements will be ignored if another recipe
    /// explicitly defines different requirements.
    pub requirements: Vec<RecipeRequirement>,
    /// This is the maximum number of times you want to 
    /// allow the recipe to be executed.
    pub max_executions: u32,
    /// This is the length of the timer that counts down 
    /// after starting a recipe. If the recipe is executed 
    /// as an Alternative Recipe, this property is not 
    /// needed, and will be ignored.
    pub warmup: u32,
    /// Defines whether or not the user is able to manually
    /// start this recipe in its verb if the requirements are met.
    pub craftable: bool,
    /// This disables the start button but still displays the 
    /// recipe when true. If the requirements for a hint recipe 
    /// and a normal, craftable recipe are met, the normal 
    /// recipe is preferred. Hintonly recipes do not need to also have craftable set to true.
    pub hint_only: bool,
    /// Defines the slot for a quick-time event that a user may
    /// insert cards into during the warmup period of this recipe.
    pub slot: Option<Slot>,
    /// A set of operations to apply to the element stack. 
    /// 
    /// When an element is added, it will be added as a 
    /// card of that element type.
    /// 
    /// When an element is removed, 
    /// the first encountered source of that element will 
    /// be removed. This could be a card of that element 
    /// or it could be a card with that element as an 
    /// aspect. 
    /// 
    /// Negative values can be used to remove 
    /// elements. If an aspect is specified for removal, 
    /// an entire card with that aspect will be removed. 
    /// 
    /// Comparisons are also available for effects; for 
    /// example, having "element_1" : "element_2" will 
    /// create as many new "element_1"s as there are 
    /// "element_2"s in the recipe stack.
    pub effects: HashMap<DefKey, ValueOperation>,
    /// A set of operations to apply to the game board. 
    /// 
    /// It will decay up to the specified amount of the element sources. 
    /// 
    /// If a specific element type is specified, it will decay to whatever 
    /// is indicated in that element's "decayTo" property, even if the
    /// element has no lifetime to cause it to decay naturally. 
    /// 
    /// If no element is specified under "decayTo", the element will be 
    /// destroyed. 
    /// 
    /// If Purge is used on an aspect, all elements with 
    /// the aspect will normally be destroyed since aspects do not 
    /// normally have a decayTo property. 
    /// 
    /// However, if for some reason 
    /// the aspect does have a decayTo property, then instead of destroying 
    /// elements with the aspect, elements with the aspect will be decayed.
    ///  
    /// This works well because the element that the aspect has been set to 
    /// decayTo will be ignored; instead, each element that has the aspect 
    /// will decay according to its own decayTo aspect. If the individual 
    /// element has no decayTo aspect, then it is destroyed.
    pub purge: HashMap<DefKey, u32>,
    /// A set of operations to apply to the recipe’s 
    /// aspect stack. The aspect stack is the sum 
    /// of all the aspects of the cards in the element 
    /// stack. The effects of this property do not 
    /// apply to the cards themselves, just to the 
    /// stack produced by their sum. This is usually 
    /// used to introduce temporary aspects for the 
    /// purpose of triggering an XTrigger reaction, 
    /// since the aspects disappear after the recipe 
    /// ends, and don't proc inductions.
    pub aspects: HashMap<DefKey, ValueOperation>,
    /// A set of deck IDs that are drawn from the specified 
    /// number of times and added to the element list at 
    /// the conclusion of the recipe.
    pub draws: HashMap<DefKey, i32>,
    /// A list of all [Mutation]s to apply to the
    /// elements in this recipe.
    pub mutations: Vec<Mutation>,
    /// Halts the specified Verb up to the given 
    /// number of different tokens. The active 
    /// recipe transitions into a closable form 
    /// that is signified by the clickable button 
    /// at the bottom left corner of the token. 
    /// Ending the recipe in the Verb will delete
    /// the Verb token and return any cards still 
    /// in the stack.
    pub halt: Option<HashMap<DefKey, u32>>,
    /// This halts the specified Verbs and deletes the 
    /// tokens at the same time, also deleting any 
    /// elements within the Verbs at the time of deletion.
    pub delete: Option<HashMap<DefKey, u32>>,
    /// The ID of an ending you’d like to trigger at the conclusion of this recipe.
    pub ending: Option<DefKey>,
    /// Audiovisual style for the warmup circle.
    pub style: WarmupStyle,
    /// All the possible [Branch]es that this element could take.
    pub branches: Vec<Branch>
}

/// A place to put a card, which could be used in
/// elements, verbs, or recipes. They act as filters,
/// only accepting cards that match their requirements.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Slot {
    /// This is the in-game representation for
    /// this deck. It is made up of the name
    /// of the deck, e.g. "slotinfluence", and the
    /// namespace it is defined in, e.g.
    /// "core.slot" -> id = "core.slot.slotinfluence"
    pub id: DefKey,
    /// This is the title text that appears above the
    /// slot as well as on the dialog produced when 
    /// the slot is clicked.
    pub label: String,
    /// This is the body text that appears appears on the
    /// dialog produced when the slot is clicked.
    pub description: String,
    /// After the recipe concludes, any element in this 
    /// slot will be destroyed. Shows the little candle 
    /// at the bottom of the slot.
    pub consumes: bool,
    /// Specific to recipe slots (Quick-time events). 
    /// When this is true, the slot will pull a qualifying 
    /// card off the table into the slot automatically, 
    /// rather than letting the player choose whether to 
    /// insert a card, or which card to insert.
    pub greedy: bool,
    /// A set of [SlotFilter]s to be applied to
    /// cards that try to be inserted into this
    /// slot.
    pub requirements: Vec<SlotFilter>,
}

/// A n element which is either required for or
/// forbidden from insertion a slot.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SlotFilter {
    /// Any card that meets any one of the properties in 
    /// Required can be put in the slot, unlike the requirements 
    /// property for recipes. Only uses positive values.

    Accept { element: DefKey, amount: u32},
    /// Any card that meets any one of a Forbidden property
    /// cannot be put in the slot, even if it qualifies for 
    /// the Required list.
    Forbid { element: DefKey, amount: u32},
}

/// A Verb is an action that can be applied
/// to an Element. Verbs defined here
/// are permanent, but a Recipe can
/// create temporary verbs that vanish once
/// their recipes resolve.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Verb {
    /// This is the in-game representation for
    /// this verb. It is made up of the name
    /// of the verb, e.g. "dream", and the
    /// namespace it is defined in, e.g.
    /// "core.verb" -> id = "core.verb.dream"
    pub id: DefKey,
    /// This is the title that appears in the 
    /// Verb UI when no card is inserted.
    pub label: String,
    /// This is the body text that appears in
    /// the verb UI when no card is inserted.
    pub description: String,
    /// Verbs can only have one [Slot].
    /// However, [Card]s can define
    /// additional slots that appear
    /// when they are inserted.
    pub slot: Option<Slot>,
}

/// Legacies define the starting conditions
/// for a game mode.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Legacy {
    /// This is the in-game representation for
    /// this legacy. It is made up of the name
    /// of the legacy, e.g. "apostlelantern", and the
    /// namespace it is defined in, e.g.
    /// "core.legacy" -> id = "core.legacy.apostlelantern"
    pub id: DefKey,
    /// The name of the Legacy as the player sees it
    pub label: String,
    /// Text displayed in the Legacy Selection screen
    pub description: String,
    /// Text displayed in the pop-up bubble at the bottom-center screen after starting the game.
    pub start_description: String,
    /// The image for the legacy displayed at the Legacy Selection screen
    pub image: String,
    /// Legacies all start with only a single [Verb] on the board, as defined here.
    pub starting_verb: DefKey,
    /// A dictionary of the [Card]s that the legacy starts with.
    pub starting_cards: HashMap<DefKey, u32>,
    /// A list of up to four [Card]s or [Aspect]s. 
    /// 
    /// These elements quantities and icons will be displayed 
    /// at the bottom of the screen for the playthrough. 
    pub status_bar_elems: Vec<DefKey>,
    /// A list of the ids of the legacies that cannot 
    /// be among the next proposed ones after this one
    pub exclude_after_legacies: Vec<DefKey>,
    /// When true, the legacy icon is added to the area where 
    /// the DLC legacies are and lets you start a new game by 
    /// clicking on the legacy icon.
    pub new_start: bool,
    /// Defining an ending here will make this legacy guaranteed to 
    /// appear after that ending; unless that ending has more than 
    /// three associated Legacies, in which case three associated 
    /// Legacies will be chosen at random from these. Only one 
    /// Ending can be defined; when one Legacy is desired to be 
    /// available from more than one Ending, look-alike Legacies 
    /// are often defined.
    pub from_ending: DefKey,
    /// This value is true for usual legacies. Setting it to false requires 
    /// the predefined ending to be achieved for the legacy to appear. 
    /// This is set to false for Apostle legacies.
    pub available_without_ending_match: bool,
}

/// Decides the music played during the ending
/// transition.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum EndingMusicKind {
    Grand,
    Melancholy,
    Vile,
}

/// Decides the color of the lights and cosmetics of the ending
/// transition.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum EndingAnimationKind {
    DramaticLight,
    DramaticLightCool,
    DramaticLightEvil,
}


/// Endings define the screen that appears
/// when the game encounters a win or loss
/// condition.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Ending {
    /// This is the in-game representation for
    /// this ending. It is made up of the name
    /// of the ending, e.g. "apostlelantern", and the
    /// namespace it is defined in, e.g.
    /// "core.ending" -> id = "core.ending.apostlelantern"
    pub id: DefKey,
    /// The name of the Ending as displayed on the Ending Screen to the player.
    pub label: String,
    // The text shown to the player on the ending screen
    pub description: String,
    /// The image shown on the left of the ending screen
    pub image: String,
    /// The kind of music that plays during the ending.
    pub music: EndingMusicKind,
    /// Decides the color of the lights and cosmetics of the ending
    /// transition.
    pub animation: EndingAnimationKind,
    /// Defines which achievement is unlocked by getting this ending.
    pub achievement: String,
}

/// XTriggers allow a mutated aspect to modify itself. 
/// These allow the aspect to transform itself, to mutate 
/// the card it belongs to, and/or spawn new cards.
/// 
/// XTriggers apply when they are in a recipe with a specific catalyst present.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Xtrigger {
    /// The default behavior of an Xtrigger. A catalyzing element is turned into
    /// another element with the specified probability. Level is used to determine
    /// the size of the new stack.
    Transform{ catalyst: DefKey, transforms_to: DefKey, amount: u32, chance: Probability },
    /// Create `amount` new cards when the catalyzing element is triggered with the specified probability.
    Spawn{ catalyst: DefKey, creates: DefKey, amount: u32, chance: Probability},
    /// Apply an aspect to a card in the specified amount with the specified probabilty.
    Mutate{ catalyst: DefKey, adds_to_catalyst: DefKey, amount: i32, chance: Probability}
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
