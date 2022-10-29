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
    let (remain, (_, id, label, description, contents)) = tuple((
        ws(tag_no_case("deck")),
        ws(defkey),
        opt(ws(string::parse)),
        opt(ws(string::parse)),
        delimited(
            ws(char('{')),
            separated_list0(line_ending, ws(deck_item)),
            ws(char('}')),
        ),
    ))(input)?;

    Ok((remain, Component::Deck(Box::new(deck_from_tokens(id, label, description, contents)?))))
}

enum DeckItem {
    Card(DefKey),
    Description(DefKey, String),
    Default(DefKey),
}

// returns (is_default, card, desc)
fn deck_item(input: &str) -> IResult<&str, (bool, DefKey, Option<String>)> {
    let (remain, (is_default, id, desc)) = tuple((
        opt(alt((
            tag("!"), 
            tag_no_case("default")
        ))),
        defkey,
        opt(pair(multispace1, string::parse)),
    ))(input)?;

    let is_default = is_default.is_some();
    let desc = desc.map(|(_, d)| d);
    Ok((remain, (is_default, id, desc)))
}

fn deck_from_tokens<I>(
    id: DefKey,
    label: Option<String>,
    description: Option<String>,
    contents: Vec<(bool, DefKey, Option<String>)>,
) -> Result<Deck, nom::Err<nom::error::Error<I>>> {
    let mut default: Option<DefKey> = None;
    let mut cards: Vec<(DefKey, Option<String>)> = Vec::new();
    let mut is_portal_deck = false;

    for (is_default, card, desc) in contents {
        if is_default {
            if default.is_some() {
                todo!("Cannot set more than one default card in a deck");
            }
            default = Some(card.clone());
        }
        if desc.is_some() {
            is_portal_deck = true;
        }
        cards.push((card.clone(), desc));
    }
    
    let label = label.unwrap_or_default();
    let description = description.unwrap_or_default();
    // If there is no Default then we must reset on exhaustion
    Ok(Deck { id, label, description, default, cards, is_portal_deck })
}
