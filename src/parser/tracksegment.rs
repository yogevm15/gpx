//! tracksegment handles parsing of GPX-spec track segments.

use std::io::Read;

use xml::reader::XmlEvent;

use crate::errors::{GpxError, GpxResult};
use crate::parser::{Context, verify_starting_tag, waypoint};
use crate::parser::extensions::WaypointExtensions;
use crate::TrackSegment;

/// consume consumes a GPX track segment from the `reader` until it ends.
pub fn consume<R: Read, E: WaypointExtensions + Default>(context: &mut Context<R, E>) -> GpxResult<TrackSegment<E>> {
    let mut segment: TrackSegment<E> = Default::default();
    verify_starting_tag(context, "trkseg")?;

    loop {
        let next_event = {
            if let Some(next) = context.reader.peek() {
                match next {
                    Ok(n) => n,
                    Err(_) => return Err(GpxError::TrackSegmentError()),
                }
            } else {
                break;
            }
        };

        match next_event {
            XmlEvent::StartElement { ref name, .. } => match name.local_name.as_ref() {
                "trkpt" => segment.points.push(waypoint::consume(context, "trkpt")?),
                child => {
                    return Err(GpxError::InvalidChildElement(
                        String::from(child),
                        "tracksegment",
                    ));
                }
            },
            XmlEvent::EndElement { ref name } => {
                if name.local_name != "trkseg" {
                    return Err(GpxError::InvalidClosingTag(
                        name.local_name.clone(),
                        "trksegment",
                    ));
                }
                context.reader.next(); //consume the end tag
                return Ok(segment);
            }
            _ => {
                context.reader.next(); //consume and ignore this event
            }
        }
    }

    Err(GpxError::MissingClosingTag("tracksegment"))
}

#[cfg(test)]
mod tests {
    use assert_approx_eq::assert_approx_eq;
    use geo::euclidean_length::EuclideanLength;

    use crate::GpxVersion;

    use super::consume;

    #[test]
    fn consume_full_trkseg() {
        let segment = consume!(
            "
            <trkseg>
                <trkpt lon=\"-77.0365\" lat=\"38.8977\">
                    <name>The White House</name>
                </trkpt>
                <trkpt lon=\"-71.063611\" lat=\"42.358056\">
                    <name>Boston, Massachusetts</name>
                </trkpt>
                <trkpt lon=\"-69.7832\" lat=\"44.31055\">
                    <name>Augusta, Maine</name>
                </trkpt>
            </trkseg>",
            GpxVersion::Gpx11
        );

        assert!(segment.is_ok());
        let segment = segment.unwrap();

        assert_eq!(segment.points.len(), 3);

        let linestring = segment.linestring();
        assert_approx_eq!(linestring.euclidean_length(), 9.2377437);
    }

    #[test]
    fn consume_empty() {
        let segment = consume!("<trkseg></trkseg>", GpxVersion::Gpx11);

        assert!(segment.is_ok());
        let segment = segment.unwrap();

        assert_eq!(segment.points.len(), 0);
    }
}
