#![allow(unused_imports)]

use anyhow::{bail, Result};
use mothlib::lantern::Attribute;
use mothlib::lantern::*;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use tracing::{event, Level};

use super::*;
use nom::{
    branch::*, bytes::complete::*, character::complete::*, combinator::*, error::*, multi::*,
    sequence::*, IResult,
};

pub fn parse(input: &str) -> IResult<&str, Component> {
    let (remain, (hidden, _, id, title, desc, aspects, decay_lifetime, statements)) = tuple((
        opt(ws(hidden)),
        ws(tag_no_case("card")),
        ws(defkey),
        ws(string::parse),
        opt(ws(string::parse)),
        ws(card_aspects),
        opt(ws(card_decays)),
        delimited(
            ws(tag("{")),
            separated_list0(line_ending, ws(card_statement)),
            ws(tag("}")),
        ),
    ))(input)?;

    let desc = desc.unwrap_or_else(|| "".to_owned());
    let (decays_to, lifetime) = decay_lifetime.unwrap_or((None, None));

    Ok((
        remain,
        Component::Card(Box::new(card_from_tokens(
            id,
            title,
            desc,
            hidden.is_some(),
            decays_to,
            lifetime,
            aspects,
            statements,
        )?)),
    ))
}

fn card_decays(input: &str) -> IResult<&str, (Option<DefKey>, Option<u32>)> {
    let (remain, (_, key, lifetime)) =
        tuple((ws(tag("->")), opt(ws(defkey)), opt(ws(u32))))(input)?;
    Ok((remain, (key, lifetime)))
}
enum CardStatement {
    Set(DefKey, json::Value),
    Induce(DefKey, Probability),
    Unique(Option<DefKey>),
    Slot(DefKey, Slot),
    Xtrigger(Xtrigger),
}

fn card_statement(input: &str) -> IResult<&str, CardStatement> {
    fn set(input: &str) -> IResult<&str, CardStatement> {
        let (remain, (_, (key, val))) = pair(
            ws(tag_no_case("set")),
            separated_pair(ws(defkey), char('='), ws(json::parse)),
        )(input)?;

        Ok((remain, CardStatement::Set(key, val)))
    }

    fn induce(input: &str) -> IResult<&str, CardStatement> {
        let (remain, (_, (key, chance))) =
            pair(ws(tag_no_case("induce")), pair(ws(defkey), ws(chance)))(input)?;

        Ok((remain, CardStatement::Induce(key, chance)))
    }

    fn unique(input: &str) -> IResult<&str, CardStatement> {
        let (remain, (_, uqgroup)) = pair(ws(tag_no_case("unique")), opt(ws(defkey)))(input)?;

        Ok((remain, CardStatement::Unique(uqgroup)))
    }

    fn xtrigger(input: &str) -> IResult<&str, CardStatement> {
        let (remain, xtrigger) = super::xtrigger(input)?;
        Ok((remain, CardStatement::Xtrigger(xtrigger)))
    }

    fn card_slot(input: &str) -> IResult<&str, CardStatement> {
        let (remain, (key, slot)) = separated_pair(defkey, ws(tag("->")), slot)(input)?;

        Ok((remain, CardStatement::Slot(key, slot)))
    }

    alt((ws(set), ws(induce), ws(unique), ws(card_slot), ws(xtrigger)))(input)
}

fn card_aspects(input: &str) -> IResult<&str, HashMap<DefKey, u32>> {
    fn card_aspect(input: &str) -> IResult<&str, (DefKey, u32)> {
        alt((
            pair(ws(defkey), success::<_, _, _>(1)),
            separated_pair(ws(defkey), ws(char(':')), ws(u32)),
        ))(input)
    }

    let (remain, aspects) = delimited(
        ws(char('(')),
        separated_list0(ws(char(',')), ws(card_aspect)),
        ws(char(')')),
    )(input)?;

    let mut map: HashMap<DefKey, u32> = HashMap::new();
    for (k, v) in aspects {
        match map.insert(k.clone(), v) {
            None => (),
            Some(_) => todo!(
                "Duplicate aspect assignment: the aspect {} has already been declared on the card.",
                k
            ),
        };
    }

    Ok((remain, map))
}

#[allow(clippy::too_many_arguments)]
fn card_from_tokens<I>(
    id: DefKey,
    title: String,
    desc: String,
    hidden: bool,
    decays_to: Option<DefKey>,
    lifetime: Option<u32>,
    aspects: HashMap<DefKey, u32>,
    statements: Vec<CardStatement>,
) -> Result<Card, nom::Err<nom::error::Error<I>>> {
    // Initialize Defaults
    let id = id;
    let label = title;
    let description = desc;
    let mut resaturate = false;
    let mut icon: Option<String> = None;
    let mut verbicon: Option<String> = None;
    let mut induces: Option<(DefKey, Probability)> = None;
    let mut unique: Option<bool> = None;
    let mut uniqueness_group: Option<DefKey> = None;
    let mut slots: HashMap<DefKey, Vec<Slot>> = HashMap::new();
    let mut xtriggers: Vec<Xtrigger> = Vec::new();
    let mut others: HashMap<DefKey, json::Value> = HashMap::new();

    for st in statements {
        match st {
            CardStatement::Set(k, v) => {
                match k.0.as_str() {
                    "id" => todo!("Failure! id cannot be set outside of the aspect signature"),
                    "label" => {
                        todo!("Failure! label cannot be set outside of the aspect signature")
                    }
                    "description" => {
                        todo!("Failure! Description cannot be set outside of the aspect signature")
                    }
                    "icon" => {
                        if let Some(old) = icon {
                            todo!("Failure! Key '{}' is already assigned with SET for this aspect: {:?}", k.0.as_str(), old)
                        } else if let json::Value::Str(s) = v {
                            icon = Some(s)
                        } else {
                            todo!(
                                "Failure! Key '{}' must be of type 'string': {:?}",
                                k.0.as_str(),
                                v
                            )
                        }
                    }
                    "verbicon" => {
                        if let Some(old) = verbicon {
                            todo!("Failure! Key '{}' is already assigned with SET for this aspect: {:?}", k.0.as_str(), old)
                        } else if let json::Value::Str(s) = v {
                            verbicon = Some(s)
                        } else {
                            todo!(
                                "Failure! Key '{}' must be of type 'string': {:?}",
                                k.0.as_str(),
                                v
                            )
                        }
                    }
                    "resaturate" => {
                        if let Some(old) = verbicon {
                            todo!("Failure! Key '{}' is already assigned with SET for this aspect: {:?}", k.0.as_str(), old)
                        } else if let json::Value::Boolean(b) = v {
                            resaturate = b;
                        } else {
                            todo!(
                                "Failure! Key '{}' must be of type 'string': {:?}",
                                k.0.as_str(),
                                v
                            )
                        }
                    }
                    _ => {
                        if let Some(old) = others.insert(k.clone(), v) {
                            todo!("Failure! Key '{}' is already assigned with SET for this aspect: {:?}", k.0.as_str(), old)
                        }
                    }
                }
            }
            CardStatement::Unique(Some(uqgroup)) => {
                if uniqueness_group.is_none() {
                    uniqueness_group = Some(uqgroup)
                } else {
                    todo!("Failure! Cannot set key 'unique <Value>' multiple times")
                }
            }
            CardStatement::Unique(None) => {
                if unique.is_none() {
                    unique = Some(true)
                } else {
                    todo!("Failure! Cannot set key 'unique' multiple times")
                }
            }
            CardStatement::Slot(verb, slotdef) => {
                match slots.get_mut(&verb) {
                    Some(entry) => {
                        entry.push(slotdef);
                    }
                    None => {
                        slots.insert(verb, vec![slotdef]);
                    }
                };
            }
            CardStatement::Induce(key, chance) => {
                if induces.is_none() {
                    induces = Some((key, chance))
                } else {
                    todo!("Failure! Cannot set key 'induce' multiple times")
                }
            }
            CardStatement::Xtrigger(xtrigger) => xtriggers.push(xtrigger),
        };
    }

    let unique = unique.unwrap_or(false);

    Ok(Card {
        id,
        label,
        description,
        icon,
        verbicon,
        induces,
        decays_to,
        hidden,
        aspects,
        lifetime,
        resaturate,
        unique,
        uniqueness_group,
        slots,
        xtriggers,
    })
}
