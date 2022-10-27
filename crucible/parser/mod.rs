use std::path::{Path, PathBuf};

use mothlib::lantern::*;
use anyhow::{Result, bail};
use pest::Parser;
use tracing::{event, Level};

#[derive(Parser)]
#[grammar = "crucible/parser/crucible.pest"]
struct CrucibleParser;

/// Parse a list of files and merge their contents
/// into a single Crucible instance.
/// 
/// This function assumes that all the paths
/// are files. It is up to the caller to ensure
/// this assumption is upheld, and this function
/// will panic otherwise.
pub fn Parse(files: Vec<PathBuf>) -> Result<Crucible> {
    let mut master = Crucible::empty();
    let mut errored = false;
    for file in files {
        match Crucible::new(file.clone()) {
            Ok(c) => match master.merge(c) {
                Ok(_) => (),
                Err(e) => {
                    errored = true;
                    event!(Level::ERROR, 
                        "Error merging \"{}\": {}", 
                        file.to_str().expect(&format!("Error: Invalid Path: {:?}", file)),
                        e
                    );
                }
            },
            Err(e) => {
                errored = true;
                event!(Level::ERROR, 
                    "Error parsing \"{}\": {}", 
                    file.to_str().expect(&format!("Error: Invalid Path: {:?}", file)),
                    e
                );
            },
        };
    }
    if errored {
        bail!("Encountered errors during Parsing")
    }
    Ok(master)
}

pub struct Crucible {
    attrs: Vec<Attribute>,
    units: Vec<Unit>,
}

impl Crucible {
    pub fn new(file: impl AsRef<Path>) -> Result<Self> {
        let raw_data = std::fs::read_to_string(file)?;
        let pdata =  CrucibleParser::parse(Rule::Crucible, &raw_data)?;

        
    
        println!("{:#?}", pdata);

        todo!()
    }

    pub fn empty() -> Self {
        Crucible{ attrs: Vec::new(), units: Vec::new() }
    }

    // Takes another Crucible instance and merges it into this one.
    pub fn merge(&mut self, other: Crucible) -> Result<()> {
        unimplemented!()
    }

    // A version of `Crucible::merge(..)` that consumes self and another,
    // returning the result. Useful syntactic sugar when chaining
    // merge calls.
    pub fn join(mut self, other: Crucible) -> Result<Crucible> {
        self.merge(other).map(|_| self)
    }
}

pub enum Unit {
    Namespace{ id: DefKey, attrs: Vec<Attribute>, units: Vec<Unit>},
    Component{ id: DefKey, attrs: Vec<Attribute>, component: Component, inherits: Option<DefKey>},
}

pub enum Component {
    Aspect(Aspect),
    Card(Card),
    Deck(Deck),
    Recipe(Recipe),
    Verb(Verb),
    Legacy(Legacy),
    Ending(Ending),
}
