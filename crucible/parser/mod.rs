use std::path::Path;
use std::path::PathBuf;
use mothlib::lantern::Attribute;
use mothlib::lantern::*;
use tracing::{event, Level};
use anyhow::{bail, Result};

use nom::{
    IResult,
    bytes::complete::*,
    character::complete::*,
    multi::*,
    sequence::*,
    branch::*,
    combinator::*,
    error::*,
};

mod string;

mod aspect;
mod card;
mod deck;
mod recipe;
mod verb;
mod legacy;
mod ending;

macro_rules! nomfail {
    ($input:expr) => {
        Err(nom::Err::Failure($input))
    };
}

#[allow(unused_macros)]
macro_rules! nomerr {
    ($input:expr) => {
        Err(nom::Err::Error($input))
    };
}

pub fn parse(files: Vec<PathBuf>) -> Result<Crucible> {
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
                        file.to_str().unwrap_or_else(|| panic!("Error: Invalid Path: {:?}", file)),
                        e
                    );
                }
            },
            Err(e) => {
                errored = true;
                event!(Level::ERROR, 
                    "Error parsing \"{}\": {}", 
                    file.to_str().unwrap_or_else(|| panic!("Error: Invalid Path: {:?}", file)),
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

#[derive(Debug)]
pub struct Crucible {
    attributes: Vec<Attribute>,
    units: Vec<Unit>,
}

impl Crucible {
    pub fn new(file: impl AsRef<Path>) -> Result<Self> {
        let raw_data = std::fs::read_to_string(file)?;
        let pdata =  crucible(raw_data)?;

        println!("{:#?}", pdata);

        todo!()
    }

    pub fn empty() -> Self {
        Crucible{ attributes: Vec::new(), units: Vec::new() }
    }

    // Takes another Crucible instance and merges it into this one.
    pub fn merge(&mut self, _other: Crucible) -> Result<()> {
        unimplemented!()
    }

    // A version of `Crucible::merge(..)` that consumes self and another,
    // returning the result. Useful syntactic sugar when chaining
    // merge calls.
    pub fn join(mut self, other: Crucible) -> Result<Crucible> {
        self.merge(other).map(|_| self)
    }
}

fn crucible(input: String) -> IResult<String, Crucible> {
    let c = context(
    "Crucible",    
    separated_pair(
        separated_list0(multispace0, global_attr), 
        multispace0,
        separated_list0(multispace0, unit)
    ))(&input);

    match c {
        Ok((remainder, (attributes, units))) => {
            if remainder.is_empty() {
                Ok((remainder.to_owned(), Crucible{attributes, units}))
            }
            else {
                nomfail!(Error::new(input, ErrorKind::NonEmpty))
            }
        },
        Err(e) => Err(e.map(|e| Error::new(e.input.to_owned(), e.code))),
    }
}

fn global_attr(input: &str) -> IResult<&str, Attribute> {
    match preceded(tag("#!"), delimited(char('['), attr, char(']')))(input) {
        Ok((r, a)) => Ok((r, a)),
        Err(e) => Err(e),
    }

}

fn local_attr(input: &str) -> IResult<&str, Attribute> {
    preceded(char('#'), delimited(char('['), attr, char(']')))(input)
}

fn attr(input: &str) -> IResult<&str, Attribute> {
    fn only_defkey(input: &str) -> IResult<&str, Attribute> {
        let (s, k) = ws(defkey)(input)?;
        Ok((s, Attribute{ key: k, value: None }))
    }
    fn defkey_value(input: &str) -> IResult<&str, Attribute> { 
        let (s, (k, v)) = separated_pair(
            ws(defkey), 
            char('='), 
            ws(value)
        )(input)?;
        Ok((s, Attribute{ key: k, value: Some(v) }))
    }
    alt((defkey_value, only_defkey))(input)
}

fn defkey(input: &str) -> IResult<&str, DefKey> {
    let (r, chrs) = take_while1(|b| { 
        matches!(b, 
            'a'..='z'
          | 'A'..='Z'
          | '0'..='9'
          | '_'
          | '-'
          | '$'
          | '.'
        )
    })(input)?;
    Ok((r, DefKey(chrs.to_owned())))
}

fn value(input: &str) -> IResult<&str, json::Value> {
    json::parse(input)
}

#[derive(Debug)]
pub enum Unit {
    Namespace{ id: DefKey, attrs: Vec<Attribute>, units: Vec<Unit>},
    Component{ id: DefKey, attrs: Vec<Attribute>, component: Component, inherits: Option<DefKey>},
}

fn unit(input: &str) -> IResult<&str, Unit> {
    alt((namespace, component))(input)
}

fn namespace(input: &str) -> IResult<&str, Unit> {
    let (remain, (attrs, _, ns_id, _, units, _)) = tuple((
            many0(ws(local_attr)),
            ws(tag_no_case("namespace")),
            ws(defkey),
            ws(char('{')),
            many0(ws(unit)),
            ws(char('}')),
        )
    )(input)?;
    Ok((remain, Unit::Namespace { id: ns_id, attrs, units }))
}

#[derive(Debug)]
pub enum Component {
    Aspect(Box<Aspect>),
    Card  (Box<Card>),
    Deck  (Box<Deck>),
    Recipe(Box<Recipe>),
    Verb  (Box<Verb>),
    Legacy(Box<Legacy>),
    Ending(Box<Ending>),
}

impl Component {
    pub fn id(&self) -> DefKey {
        match self {
            Component::Aspect(c)  => c.id.clone(),
            Component::Card(c)    => c.id.clone(),
            Component::Deck(c)    => c.id.clone(),
            Component::Recipe(c)  => c.id.clone(),
            Component::Verb(c)    => c.id.clone(),
            Component::Legacy(c)  => c.id.clone(),
            Component::Ending(c)  => c.id.clone(),
        }
    }
}

fn component(input: &str) -> IResult<&str, Unit> {
    fn component_inner(input: &str) -> IResult<&str, Component> {
        alt((
            aspect::parse,
            card::parse,
            deck::parse,
            recipe::parse,
            verb::parse,
            legacy::parse,
            ending::parse,
        ))(input)
    }
    let (remain, (attrs, inherits, component_inner)) = tuple((
        many0(ws(local_attr)),
        opt(ws(inherit)),
        ws(component_inner),
    ))(input)?;
    Ok((remain, Unit::Component{ id: component_inner.id(), attrs, component: component_inner, inherits }))
}

fn inherit(input: &str) -> IResult<&str, DefKey> {
    let (remain, (_, key)) = pair(ws(tag_no_case("from")), ws(defkey))(input)?;
    Ok((remain, key))
}

fn hidden(input: &str) -> IResult<&str, ()> {
    let (remain, _) = alt((
        tag("hidden"),
        tag("?")
    ))(input)?;
    Ok((remain, ()))
}

fn xtrigger(input: &str) -> IResult<&str, Xtrigger> {
    enum XtriggerKind {
        Transform{ target: DefKey, amount: u32, chance: Probability },
        Spawn{ target: DefKey, amount: u32, chance: Probability },
        Mutate{ target: DefKey, amount: i32, chance: Probability },
    }
    pub fn spawn(input: &str) -> IResult<&str, XtriggerKind> {
        let (remain, (_, _, target, _, amount, _, chance, _)) = tuple((
            ws(tag_no_case("spawn")),
            multispace0,
            defkey,
            char(':'),
            u32,
            multispace0,
            opt(chance),
            multispace0,
        ))(input)?;

        let chance = chance.unwrap_or(Probability::new(100).unwrap());
        Ok((remain, XtriggerKind::Spawn{target, amount, chance}))
    }
    pub fn mutate(input: &str) -> IResult<&str, XtriggerKind> {
        let (remain, (_, _, target, _, amount, _, chance, _)) = tuple((
            ws(tag_no_case("mutate")),
            multispace0,
            defkey,
            char(':'),
            i32,
            multispace0,
            opt(chance),
            multispace0,
        ))(input)?;

        let chance = chance.unwrap_or(Probability::new(100).unwrap());
        Ok((remain, XtriggerKind::Mutate{target, amount, chance}))
    }
    pub fn transform(input: &str) -> IResult<&str, XtriggerKind> {
        let (remain, (_, target, _, amount, _, chance, _)) = tuple((
            multispace0,
            defkey,
            char(':'),
            u32,
            multispace0,
            opt(chance),
            multispace0,
        ))(input)?;

        let chance = chance.unwrap_or(Probability::new(100).unwrap());
        Ok((remain, XtriggerKind::Transform{target, amount, chance}))
    }
    pub fn basic(input: &str) -> IResult<&str, XtriggerKind> {
        let (remain, (target, chance)) = tuple((
            ws(defkey),
            opt(ws(chance)),
        ))(input)?;

        let chance = chance.unwrap_or(Probability::new(100).unwrap());
        Ok((remain, XtriggerKind::Transform{ target, amount: 1, chance}))
    }

    let (remain, (_, catalyst, _, trigger_inner)) = tuple((
        ws(tag("xtrigger")),
        ws(defkey),
        ws(tag("->")),
        alt((
            ws(spawn),
            ws(mutate),
            ws(transform),
            ws(basic),
        )),
    ))(input)?;

    let trigger = match trigger_inner {
        XtriggerKind::Transform { target, amount, chance } => Xtrigger::Transform { 
            catalyst, 
            transforms_to: target, 
            amount, 
            chance
        },
        XtriggerKind::Spawn { target, amount, chance } => Xtrigger::Spawn { 
            catalyst, 
            creates: target, 
            amount,
            chance
        },
        XtriggerKind::Mutate { target, amount, chance } => Xtrigger::Mutate { 
            catalyst, 
            adds_to_catalyst: target, 
            amount, 
            chance
        },
    };

    Ok((remain, trigger))
}

/// Parses a single SlotDef. Does not parse predicates, such
/// as the verbs in a card's slot def.
fn slot(input: &str) -> IResult<&str, Slot> {
    // returns (isConsume, isGreedy)
    pub fn slotkind(input: &str) -> IResult<&str, (Option<()>, Option<()>)> {
        let (remain, (isConsume, isGreedy)) = alt((
            permutation((ws(tag("!")), ws(tag("?")))),
            permutation((ws(tag_no_case("consume")), ws(tag_no_case("greedy")))),
            pair(success::<_,_,_>(""), ws(tag("!"))),
            pair(ws(tag("?")), success::<_,_,_>("")),
            pair(success::<_,_,_>(""), ws(tag_no_case("consume"))),
            pair(ws(tag_no_case("greedy")), success::<_,_,_>("")),
        ))(input)?;

        let isConsume = match isConsume.to_lowercase().as_str() {
            "!" | "consume" => Some(()),
            _ => None,
        };

        let isGreedy = match isGreedy.to_lowercase().as_str() {
            "?" | "greedy" => Some(()),
            _ => None,
        };

        Ok((remain, (isConsume, isGreedy)))
    }
    pub fn slotfilter(input: &str) -> IResult<&str, SlotFilter> {
        let (remain, (forbid, element, _, amount)) = tuple((
            opt(char('!')),
            defkey,
            char(':'),
            u32,
        ))(input)?;

        let filter = match forbid {
            Some(_) => SlotFilter::Accept { element, amount },
            None => SlotFilter::Forbid { element, amount },
        };
        Ok((remain, filter))
    }

    let (remain, (kind, _, id, label, description, requirements)) = tuple((
        opt(ws(slotkind)),
        ws(tag_no_case("slot")),
        ws(defkey),
        ws(string::parse),
        ws(string::parse),
        opt(
            delimited(
                ws(char('(')), 
                separated_list0(char(','), slotfilter), 
                ws(char(')')))
        )
    ))(input)?;

    let mut consumes = false;
    let mut greedy = false;
    if let Some((isConsume, isGreedy)) = kind {
        consumes = isConsume.is_some();
        greedy = isGreedy.is_some();
    }
    let requirements = requirements.unwrap_or_else(|| Vec::new() );

    Ok((remain, Slot{ id, label, description, consumes, greedy, requirements }))
}

/// A combinator that takes a parser `inner` and produces a parser that also consumes both leading and 
/// trailing whitespace, returning the output of `inner`.
fn ws<'a, F: 'a, O, E: ParseError<&'a str>>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
  where
  F: Fn(&'a str) -> IResult<&'a str, O, E>,
{
  delimited(
    multispace0,
    inner,
    multispace0
  )
}

fn comment<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, (), E>
{
    let (remainder, (_slashes, _comment)) = pair(tag("//"), is_not("\n\r"))(i)?;
    Ok((remainder, ()))
}

fn block_comment<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, (), E> {
    let (remainder, (_open, _comment, _close)) = tuple((tag("(*"), take_until("*)"), tag("*)")))(i)?;
    Ok((remainder, ()))
}

fn chance(input: &str) -> IResult<&str, Probability> {
    let (remain, (num, _)) = pair(
        verify(u8, |num| matches!(num, 0..=100)),
        opt(char('%'))
    )(input)?;
    // The verify() combinator above makes sure the value
    // is OK, so we can just unwrap here
    Ok((remain, Probability::new(num).expect("Parser failed to uphold Probability invariant!")))
}