#![allow(unused_imports)]

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
    let (remain, (_, id, label, description, slot)) = tuple((
        ws(tag_no_case("verb")),
        ws(defkey),
        ws(string::parse),
        ws(string::parse),
        opt(delimited(
            ws(char('(')),
            ws(slot),
            ws(char(')')),
        )),
    ))(input)?;

    Ok((remain, Component::Verb(Box::new(Verb{ id, label, description, slot }))))
}