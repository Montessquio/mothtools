#![allow(unused_imports)]

use anyhow::{bail, Result};
use mothlib::lantern::Attribute;
use mothlib::lantern::*;
use std::borrow::Borrow;
use std::path::Path;
use std::path::PathBuf;
use tracing::{event, Level};

use super::*;
use nom::{
    branch::*, bytes::complete::*, character::complete::*, combinator::*, error::*, multi::*,
    sequence::*, IResult,
};

pub fn parse(input: &str) -> IResult<&str, Component> {
    let (remain, (_, id, label, description, content)) =
        tuple((
            ws(tag_no_case("ending")), 
            ws(defkey), 
            ws(string::parse),
            ws(string::parse),
            ws(json::parse)))(input)?;

    let mut content = match content {
        json::Value::Object(o) => Ok(o),
        _ => todo!("The content of an 'ending' must be a JSON dictionary!"),
    }?;

    let image = if let Some(v) = content.remove("image") {
        match v {
            json::Value::Str(s) => s,
            _ => todo!("Key 'image' must have a value of type String, was {:?}", v)
        }
    }
    else {
        todo!("Key 'image' is required")
    };

    let music = if let Some(v) = content.remove("flavour") {
        match v {
            json::Value::Str(s) => match s.to_lowercase().as_str() {
                "grand" => EndingMusicKind::Grand,
                "melancholy" => EndingMusicKind::Melancholy,
                "vile" => EndingMusicKind::Vile,
                _ => todo!("Value for key 'anim' must be 'grand', 'melancholy', or 'vile'."),
            },
            _ => todo!("Key 'flavour' must have a value of type String, was {:?}", v)
        }
    }
    else {
        // Default
        EndingMusicKind::Grand
    };

    let animation = if let Some(v) = content.remove("anim") {
        match v {
            json::Value::Str(s) => match s.to_lowercase().as_str() {
                "dramaticlight" => EndingAnimationKind::DramaticLight,
                "dramagiclightcool" => EndingAnimationKind::DramaticLightCool,
                "dramaticlightevil" => EndingAnimationKind::DramaticLightEvil,
                _ => todo!("Value for key 'anim' must be 'dramaticlight', 'dramagiclightcool', or 'dramaticlightevil'."),
            },
            _ => todo!("Key 'anim' must have a value of type String, was {:?}", v)
        }
    }

    else {
        // Default
        EndingAnimationKind::DramaticLight
    };

    let achievement = if let Some(v) = content.remove("achievementid") {
        match v {
            json::Value::Str(s) => s,
            _ => todo!("Key 'achievementid' must have a value of type String, was {:?}", v)
        }
    }
    else {
        // Default
        "XXX".to_owned()
    };

    // convert JSON tag to Lantern struct

    Ok((
        remain,
        Component::Ending(Box::new(Ending {
            id,
            label,
            description,
            image,
            music,
            animation,
            achievement,
        })),
    ))
}
