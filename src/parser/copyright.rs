//! Copyright handles parsing of GPX-spec copyright.

use std::io::Read;

use xml::reader::XmlEvent;

use crate::errors::{GpxError, GpxResult};
use crate::GpxCopyright;
use crate::parser::{Context, string, verify_starting_tag};
use crate::parser::extensions::WaypointExtensions;

/// consume consumes a GPX copyright from the `reader` until it ends.
/// When it returns, the reader will be at the element after the end GPX copyright tag.
pub fn consume<R: Read, E: WaypointExtensions + Default>(context: &mut Context<R, E>) -> GpxResult<GpxCopyright> {
    let mut copyright: GpxCopyright = Default::default();
    let attributes = verify_starting_tag(context, "copyright")?;
    let attr = attributes
        .into_iter()
        .find(|attr| attr.name.local_name == "author");

    copyright.author = attr.map(|a| a.value);

    loop {
        let next_event = {
            if let Some(next) = context.reader.peek() {
                match next {
                    Ok(n) => n,
                    Err(_) => return Err(GpxError::EventParsingError("copyright")),
                }
            } else {
                break;
            }
        };

        match next_event {
            XmlEvent::StartElement { ref name, .. } => match name.local_name.as_ref() {
                "license" => copyright.license = Some(string::consume(context, "license", false)?),
                "year" => copyright.year = string::consume(context, "year", false)?.parse().ok(),
                child => {
                    return Err(GpxError::InvalidChildElement(
                        String::from(child),
                        "copyright",
                    ));
                }
            },
            XmlEvent::EndElement { ref name } => {
                if name.local_name != "copyright" {
                    return Err(GpxError::InvalidClosingTag(
                        name.local_name.clone(),
                        "copyright",
                    ));
                }
                context.reader.next();
                return Ok(copyright);
            }
            _ => {
                context.reader.next(); //consume and ignore this event
            }
        }
    }

    Err(GpxError::MissingClosingTag("copyright"))
}

#[cfg(test)]
mod tests {
    use crate::GpxVersion;

    use super::consume;

    #[test]
    fn consume_simple_copyright() {
        let copyright = consume!(
            "<copyright author='OpenStreetMap contributors'><year>2020</year><license>https://www.openstreetmap.org/copyright</license></copyright>",
            GpxVersion::Gpx11
        );

        assert!(copyright.is_ok());

        let copyright = copyright.unwrap();

        assert!(copyright.author.is_some());
        assert_eq!(copyright.author.unwrap(), "OpenStreetMap contributors");

        assert!(copyright.year.is_some());
        assert_eq!(copyright.year.unwrap(), 2020);

        assert!(copyright.license.is_some());
        assert_eq!(
            copyright.license.unwrap(),
            "https://www.openstreetmap.org/copyright"
        );
    }

    #[test]
    fn consume_barebones() {
        let copyright = consume!(
            "<copyright author='pelmers'></copyright>",
            GpxVersion::Gpx11
        );

        assert!(copyright.is_ok());
        let copyright = copyright.unwrap();

        assert_eq!(copyright.author.unwrap(), "pelmers");
    }
}
