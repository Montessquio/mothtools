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
    todo!()
}
