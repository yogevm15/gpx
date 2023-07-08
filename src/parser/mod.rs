//! Handles parsing GPX format.

use std::io::Read;
use std::iter::Peekable;
use std::marker::PhantomData;

use xml::{EventReader, ParserConfig};
use xml::attribute::OwnedAttribute;
use xml::reader::{Events, XmlEvent};

use crate::errors::{GpxError, GpxResult};
use crate::parser::extensions::WaypointExtensions;
use crate::types::GpxVersion;

// Just a shared macro for testing 'consume'.
#[cfg(test)]
#[macro_export]
macro_rules! consume {
    ($xml:expr, $version:expr) => {{
        use std::io::BufReader;
        use $crate::parser::create_context;
        consume(&mut create_context(
            BufReader::new($xml.as_bytes()),
            $version,
        ))
    }};
    ($xml:expr, $version:expr, $tagname:expr) => {{
        use crate::parser::create_context;
        use std::io::BufReader;
        consume(
            &mut create_context(BufReader::new($xml.as_bytes()), $version),
            $tagname,
        )
    }};
    ($xml:expr, $version:expr, $tagname:expr, $allow_empty:expr) => {{
        use crate::parser::create_context;
        use std::io::BufReader;
        consume(
            &mut create_context(BufReader::new($xml.as_bytes()), $version),
            $tagname,
            $allow_empty,
        )
    }};
}

pub mod bounds;
pub mod copyright;
pub mod email;
pub mod extensions;
pub mod fix;
pub mod gpx;
pub mod link;
pub mod metadata;
pub mod person;
pub mod route;
pub mod string;
pub mod time;
pub mod track;
pub mod tracksegment;
pub mod waypoint;

pub struct Context<R: Read, E: WaypointExtensions + Default> {
    reader: Peekable<Events<R>>,
    version: GpxVersion,
    phantom: PhantomData<E>,
}

impl<R: Read, E: WaypointExtensions + Default> Context<R, E> {
    pub fn new(reader: Peekable<Events<R>>, version: GpxVersion) -> Context<R, E> {
        Context { reader, version, phantom: Default::default() }
    }

    pub fn reader(&mut self) -> &mut Peekable<Events<R>> {
        &mut self.reader
    }

    pub fn consume_waypoint_extensions(&mut self) -> GpxResult<E::ExtensionsValue> {
        E::consume(self)
    }
}

pub fn verify_starting_tag<R: Read, E: WaypointExtensions + Default>(
    context: &mut Context<R, E>,
    local_name: &'static str,
) -> Result<Vec<OwnedAttribute>, GpxError> {
    //makes sure the specified starting tag is the next tag on the stream
    //we ignore and skip all xmlevents except StartElement, Characters and EndElement
    loop {
        let next = context.reader.next();
        match next {
            Some(Ok(XmlEvent::StartElement {
                        name, attributes, ..
                    })) => {
                if name.local_name != local_name {
                    return Err(GpxError::InvalidChildElement(name.local_name, local_name));
                } else {
                    return Ok(attributes);
                }
            }
            Some(Ok(XmlEvent::EndElement { name, .. })) => {
                return Err(GpxError::InvalidChildElement(name.local_name, local_name));
            }
            Some(Ok(XmlEvent::Characters(chars))) => {
                return Err(GpxError::InvalidChildElement(chars, local_name));
            }
            Some(_) => {} //ignore other elements
            None => return Err(GpxError::MissingOpeningTag(local_name)),
        }
    }
}

pub(crate) fn create_context<R: Read, E: WaypointExtensions + Default>(reader: R, version: GpxVersion) -> Context<R, E> {
    let parser_config = ParserConfig {
        whitespace_to_characters: true, //convert Whitespace event to Characters
        cdata_to_characters: true,      //convert CData event to Characters
        ..ParserConfig::new()
    };
    let parser = EventReader::new_with_config(reader, parser_config);
    let events = parser.into_iter().peekable();
    Context::new(events, version)
}
