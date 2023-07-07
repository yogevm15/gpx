//! Reads an activity from GPX format.

use std::io::Read;

use crate::{Gpx, GpxVersion};
use crate::errors::GpxResult;
use crate::parser::{create_context, gpx};
use crate::parser::extensions::{EmptyExtensions, WaypointExtensions};

/// Reads an activity in GPX format.
///
/// Takes any `std::io::Read` as its reader, and returns a
/// `Result<Gpx>`.
///
/// ```
/// use std::io::BufReader;
/// use gpx::read;
/// use gpx::Gpx;
/// use gpx::errors::GpxError;
///
/// // You can give it anything that implements `std::io::Read`.
/// let data = BufReader::new("<gpx></gpx>".as_bytes());
///
/// let res: Result<Gpx, GpxError> = read(data);
///
/// match res {
///     Ok(gpx) => {
///         // ..
///     }
///
///     Err(e) => {
///         // ..
///     }
/// }
/// ```
pub fn read<R: Read>(reader: R) -> GpxResult<Gpx<EmptyExtensions>> {
    read_with_extensions::<R, EmptyExtensions>(reader)
}


pub fn read_with_extensions<R: Read, E: WaypointExtensions + Default>(reader: R) -> GpxResult<Gpx<E>> {
    gpx::consume(&mut create_context::<R, E>(reader, GpxVersion::Unknown))
}
