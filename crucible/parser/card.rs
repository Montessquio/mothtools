#![allow(unused_imports)]

use std::collections::HashMap;
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
use super::*;

pub fn parse(input: &str) -> IResult<&str, Component> {
    let (remain, (hidden, _, id, title, desc, aspects, decays_to, statements)) = tuple((
        opt(ws(hidden)),
        ws(tag_no_case("card")),
        ws(defkey),
        ws(string::parse),
        opt(ws(string::parse)),
        ws(card_aspects),
        opt(ws(aspect_decays)),
        delimited(
            ws(tag("{")),
            separated_list0(line_ending, ws(card_statement)),
            ws(tag("}"))
        ),
    ))(input)?; 

    Ok((remain, Component::Card(Box::new(card_from_tokens(id, title, desc, hidden.is_some(), decays_to, statements)?))))
}

fn card_decays(input: &str) -> IResult<&str, (Option<DefKey>, Option<u32>)> {
    let (remain, (_, key, lifetime)) = tuple((
        ws(tag("->")), 
        opt(ws(defkey)),
        opt(ws(u32)),
    ))(input)?;
    Ok((remain, (key, lifetime)))
}
enum CardStatement {
    Set(DefKey, json::Value),
    Induce(DefKey, Probability),
    Unqiue(Option<DefKey>),
    Slot(DefKey, Slot),
    Xtrigger(Xtrigger),
}

fn card_statement(input: &str) -> IResult<&str, CardStatement> {
    fn set(input: &str) -> IResult<&str, CardStatement> {
        let (remain, (_, (key, val))) = pair(ws(tag_no_case("set")),
        separated_pair(
            ws(defkey),
            char('='),
            ws(json::parse)
        ))(input)?;

        Ok((remain, CardStatement::Set(key, val)))
    }

    fn induce(input: &str) -> IResult<&str, CardStatement> {
        let (remain, (_, (key, chance))) = pair(ws(tag_no_case("induce")),
        pair(
            ws(defkey),
            ws(chance)
        ))(input)?;

        Ok((remain, CardStatement::Induce(key, chance)))
    }

    fn unique(input: &str) -> IResult<&str, CardStatement> {
       todo!() 
    }
 
    fn xtrigger(input: &str) -> IResult<&str, CardStatement> {
        let (remain, xtrigger) = super::xtrigger(input)?;
        Ok((remain, CardStatement::Xtrigger(xtrigger)))
    }

    fn card_slot(input: &str) -> IResult<&str, CardStatement> {
    let (remain, xtrigger) = slot(input)?;
        Ok((remain, CardStatement::Slot((), ())))
    }

    alt((
        ws(set),
        ws(induce),
        ws(unique),
        ws(card_slot),
        ws(xtrigger),
    ))(input)
}

fn aspect_from_tokens<I>(id: DefKey, title: String, desc: String, hidden: bool, decays_to: Option<DefKey>, statements: Vec<AspectStatement>) -> Result<Aspect, nom::Err<nom::error::Error<I>>> {
    // Initialize Defaults
    let id = id;
    let label = title;
    let description = desc;
    let mut icon: Option<String> = None;
    let mut verbicon: Option<String> = None;
    let mut induces: Option<(DefKey, Probability)> = None;
    let mut xtriggers: Vec<Xtrigger> = Vec::new();
    let mut others: HashMap<DefKey, json::Value> = HashMap::new();

    for st in statements {
        match st {
            AspectStatement::Set(k, v) => {
                match k.0.as_str() {
                    "id" => todo!("Failure! id cannot be set outside of the aspect signature"),
                    "label" => todo!("Failure! label cannot be set outside of the aspect signature"),
                    "description" => todo!("Failure! Description cannot be set outside of the aspect signature"),
                    "icon" => {
                        if let Some(old) = icon {
                            todo!("Failure! Key '{}' is already assigned with SET for this aspect: {:?}", k.0.as_str(), old)
                        }
                        else if let json::Value::Str(s) = v {
                            icon = Some(s)
                        }
                        else {
                            todo!("Failure! Key '{}' must be of type 'string': {:?}", k.0.as_str(), v)
                        }
                    },
                    "verbicon" => {
                        if let Some(old) = verbicon {
                            todo!("Failure! Key '{}' is already assigned with SET for this aspect: {:?}", k.0.as_str(), old)
                        }
                        else if let json::Value::Str(s) = v {
                            verbicon = Some(s)
                        }
                        else {
                            todo!("Failure! Key '{}' must be of type 'string': {:?}", k.0.as_str(), v)
                        }
                    },
                    _ => if let Some(old) = others.insert(k.clone(), v) {
                        todo!("Failure! Key '{}' is already assigned with SET for this aspect: {:?}", k.0.as_str(), old)
                    },
                }
            },
            AspectStatement::Induce(key, chance) => {
                if induces.is_none() {
                    induces = Some((key, chance))
                }
                else {
                    todo!("Failure! Cannot set key 'induce' multiple times")
                }
            },
            AspectStatement::Xtrigger(xtrigger) => xtriggers.push(xtrigger),
        };
    }

    Ok(Aspect{id, label, description, icon, verbicon, induces, decays_to, hidden, xtriggers, others})
}