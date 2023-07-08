// This is a pretty complete functional test of the library.
// Feel free to read through these tests and their accompanying
// .gpx files to see how usage might be.

use std::error::Error;
use std::fs::File;
use std::io::BufReader;

use assert_approx_eq::assert_approx_eq;
use geo::algorithm::haversine_distance::HaversineDistance;
use geo::euclidean_length::EuclideanLength;
use geo_types::{Geometry, Point};
use time::{Date, Month, PrimitiveDateTime, Time};

use gpx::{Fix, read};

#[test]
fn gpx_reader_read_test_badxml() {
    // Should fail with badly formatted XML.
    let file = File::open("tests/fixtures/badcharacter.xml").unwrap();
    let reader = BufReader::new(file);

    let result = read(reader);

    assert!(result.is_err());
}

#[test]
fn gpx_reader_read_test_wikipedia() {
    // Should not give an error, and should have all the correct data.
    let file = File::open("tests/fixtures/wikipedia_example.gpx").unwrap();
    let reader = BufReader::new(file);

    let result = read(reader);
    assert!(result.is_ok());

    let result = result.unwrap();

    // Check the metadata, of course; here it has a time.
    let metadata = result.metadata.unwrap();
    let expect = PrimitiveDateTime::new(
        Date::from_calendar_date(2009, Month::October, 17).unwrap(),
        Time::from_hms(22, 58, 43).unwrap(),
    )
        .assume_utc()
        .into();

    assert_eq!(metadata.time.unwrap(), expect);

    assert_eq!(metadata.links.len(), 1);
    let link = &metadata.links[0];
    assert_eq!(link.href, "http://www.garmin.com");
    assert_eq!(link.text, Some(String::from("Garmin International")));

    // There should just be one track, "example gpx document".
    assert_eq!(result.tracks.len(), 1);
    let track = &result.tracks[0];

    assert_eq!(track.name, Some(String::from("Example GPX Document")));

    // Each point has its own information; test elevation.
    assert_eq!(track.segments.len(), 1);
    let points = &track.segments[0].points;

    assert_eq!(points.len(), 3);
    assert_eq!(points[0].elevation, Some(4.46));
    assert_eq!(points[1].elevation, Some(4.94));
    assert_eq!(points[2].elevation, Some(6.87));
}

#[test]
fn gpx_reader_read_test_gpsies() {
    // Should not give an error, and should have all the correct data.
    let file = File::open("tests/fixtures/gpsies_example.gpx").unwrap();
    let reader = BufReader::new(file);

    let result = read(reader);
    match result {
        Ok(_) => {}
        Err(ref e) => {
            println!("{:?}", e);
        }
    }
    assert!(result.is_ok());

    let result = result.unwrap();

    // Check the metadata, of course; here it has a time.
    let metadata = result.metadata.unwrap();

    let expect = PrimitiveDateTime::new(
        Date::from_calendar_date(2019, Month::September, 11).unwrap(),
        Time::from_hms(17, 8, 31).unwrap(),
    )
        .assume_utc()
        .into();

    assert_eq!(metadata.time.unwrap(), expect);

    assert_eq!(metadata.links.len(), 1);
    let link = &metadata.links[0];
    assert_eq!(link.href, "https://www.gpsies.com/");
    assert_eq!(link.text, Some(String::from("Innrunde on AllTrails")));

    // There should just be one track, "example gpx document".
    assert_eq!(result.tracks.len(), 1);
    let track = &result.tracks[0];

    assert_eq!(track.name, Some(String::from("Innrunde on AllTrails")));

    let link = &result.tracks[0].links[0];

    assert_eq!(link.href, "https://www.gpsies.com/map.do");

    // Each point has its own information; test elevation.
    assert_eq!(track.segments.len(), 1);
    let points = &track.segments[0].points;

    assert_eq!(points[0].elevation, Some(305.0));
    assert_eq!(points[1].elevation, Some(304.0));
    assert_eq!(points[2].elevation, Some(305.0));
}

#[test]
fn gpx_reader_read_test_empty_elevation() {
    let file = File::open("tests/fixtures/wahoo_example.gpx").unwrap();
    let reader = BufReader::new(file);

    let result = read(reader);
    assert!(result.is_ok());
    let res = result.unwrap();

    // Test for every single point in the file.
    for track in &res.tracks {
        for segment in &track.segments {
            for point in &segment.points {
                let elevation = point.elevation.is_none();
                assert!(elevation);
            }
        }
    }
}

#[test]
fn gpx_reader_read_test_garmin_activity() {
    let file = File::open("tests/fixtures/garmin-activity.gpx").unwrap();
    let reader = BufReader::new(file);

    let result = read(reader);
    assert!(result.is_ok());
    let res = result.unwrap();

    // Check the info on the metadata.
    let metadata = res.metadata.unwrap();

    let expect = PrimitiveDateTime::new(
        Date::from_calendar_date(2017, Month::July, 29).unwrap(),
        Time::from_hms(14, 46, 35).unwrap(),
    )
        .assume_utc()
        .into();

    assert_eq!(metadata.time.unwrap(), expect);

    assert_eq!(metadata.links.len(), 1);
    let link = &metadata.links[0];
    assert_eq!(link.text, Some(String::from("Garmin Connect")));
    assert_eq!(link.href, String::from("connect.garmin.com"));

    // Check the main track.
    assert_eq!(res.tracks.len(), 1);
    let track = &res.tracks[0];

    assert_eq!(track.name, Some(String::from("casual stroll")));
    assert_eq!(track.type_, Some(String::from("running")));

    // Check some Geo operations on the track.
    let mls = track.multilinestring();
    assert_approx_eq!(mls.euclidean_length(), 0.12704048);

    // Get the first track segment.
    assert_eq!(track.segments.len(), 1);
    let segment = &track.segments[0];

    // Test for every single point in the file.
    for point in segment.points.iter() {
        // Elevation is between 90 and 220.
        let elevation = point.elevation.unwrap();
        assert!(elevation > 90. && elevation < 220.);

        // All the points should be close (5000 units, its closer than you think).
        let reference_point = Point::new(-121.97, 37.24);
        let distance = reference_point.haversine_distance(&point.point());
        assert!(distance < 5000.);

        // Time is between a day before and after.
        let time = point.time.unwrap();

        let before = PrimitiveDateTime::new(
            Date::from_calendar_date(2017, Month::July, 28).unwrap(),
            Time::from_hms(0, 0, 0).unwrap(),
        )
            .assume_utc()
            .into();

        let after = PrimitiveDateTime::new(
            Date::from_calendar_date(2017, Month::July, 30).unwrap(),
            Time::from_hms(0, 0, 0).unwrap(),
        )
            .assume_utc()
            .into();

        assert!(time > before);
        assert!(time < after);

        // Should coerce to Point.
        let geo: Geometry<f64> = point.clone().into();
        match geo {
            Geometry::Point(_) => {} // ok
            _ => panic!("point.into() gave bad geometry"),
        }

        // It's missing almost all fields, actually.
        assert!(point.name.is_none());
        assert!(point.comment.is_none());
        assert!(point.description.is_none());
        assert!(point.source.is_none());
        assert!(point.symbol.is_none());
        assert!(point.type_.is_none());
        assert_eq!(point.links.len(), 0);
    }
}

#[test]
fn gpx_reader_read_test_lovers_lane() {
    let file = File::open("tests/fixtures/ecology-trail-and-lovers-lane-loop.gpx").unwrap();
    let reader = BufReader::new(file);

    let result = read(reader);
    assert!(result.is_ok());
    let res = result.unwrap();

    // Check the info on the metadata.
    let metadata = res.metadata.unwrap();

    assert_eq!(metadata.name, Some(String::from("Trail Planner Map")));
    assert_eq!(metadata.links.len(), 1);
    let link = &metadata.links[0];
    assert_eq!(
        link.text,
        Some(String::from("Trail Planner Map on AllTrails"))
    );
    assert_eq!(link.href, String::from("https://www.gpsies.com/"));

    // Check the main track.
    let routes = &res.routes;
    assert_eq!(
        routes[0].name,
        Some(String::from("Trail Planner Map on AllTrails"))
    );
    assert_eq!(routes[0].points.len(), 139);

    // Test for every single point in the file.
    for point in routes[0].points.iter() {
        // Elevation is between 15 and 100
        let elevation = point.elevation.unwrap();
        assert!(elevation > 15. && elevation < 100.);

        // Should coerce to Point.
        let geo: Geometry<f64> = point.clone().into();
        match geo {
            Geometry::Point(_) => {} // ok
            _ => panic!("point.into() gave bad geometry"),
        }

        // It's missing almost all fields, actually.
        assert!(point.name.is_none());
        assert!(point.comment.is_none());
        assert!(point.description.is_none());
        assert!(point.source.is_none());
        assert!(point.symbol.is_none());
        assert!(point.type_.is_none());
        assert_eq!(point.links.len(), 0);
    }
}

#[test]
fn gpx_reader_read_test_with_accuracy() {
    let file = File::open("tests/fixtures/with_accuracy.gpx").unwrap();
    let reader = BufReader::new(file);

    let result = read(reader);
    assert!(result.is_ok());
    let res = result.unwrap();

    // Check the info on the metadata.
    let metadata = res.metadata.unwrap();
    assert_eq!(metadata.name.unwrap(), "20170412_CARDIO.gpx");

    assert_eq!(metadata.links.len(), 0);

    // Check the main track.
    assert_eq!(res.tracks.len(), 1);
    let track = &res.tracks[0];

    assert_eq!(track.name, Some(String::from("Cycling")));

    // Get the first track segment.
    assert_eq!(track.segments.len(), 1);
    let segment = &track.segments[0];

    // Get the first point
    assert_eq!(segment.points.len(), 3);
    let points = &segment.points;

    assert_eq!(points[0].fix, Some(Fix::DGPS));
    assert_eq!(points[0].sat.unwrap(), 4);
    assert_eq!(points[0].hdop.unwrap(), 5.);
    assert_eq!(points[0].vdop.unwrap(), 6.2);
    assert_eq!(points[0].pdop.unwrap(), 728.);
    assert_eq!(points[0].dgps_age.unwrap(), 1.);
    assert_eq!(points[0].dgpsid.unwrap(), 3);

    assert_eq!(points[1].fix, Some(Fix::ThreeDimensional));
    assert_eq!(points[1].sat.unwrap(), 5);
    assert_eq!(points[1].hdop.unwrap(), 3.6);
    assert_eq!(points[1].vdop.unwrap(), 5.);
    assert_eq!(points[1].pdop.unwrap(), 619.1);
    assert_eq!(points[1].dgps_age.unwrap(), 2.01);
    assert_eq!(points[1].dgpsid.unwrap(), 4);

    assert_eq!(
        points[2].fix,
        Some(Fix::Other("something_not_in_the_spec".to_string()))
    );
}

#[test]
fn gpx_reader_read_test_strava_route() {
    let file = File::open("tests/fixtures/strava_route_example.gpx").unwrap();
    let reader = BufReader::new(file);

    let result = read(reader);
    assert!(result.is_ok());
    let res = result.unwrap();

    // Check the info on the metadata.
    let metadata = res.metadata.unwrap();
    assert_eq!(metadata.name.unwrap(), "Afternoon Run");
    let copyright = metadata.copyright.unwrap();
    assert_eq!(copyright.author.unwrap(), "OpenStreetMap contributors");
    assert_eq!(copyright.year.unwrap(), 2020);
    assert_eq!(
        copyright.license.unwrap(),
        "https://www.openstreetmap.org/copyright"
    );

    assert_eq!(metadata.links.len(), 1);

    // Check the main track.
    assert_eq!(res.tracks.len(), 1);
    let track = &res.tracks[0];
    assert_eq!(track.segments.len(), 1);
    let segment = &track.segments[0];
    assert_eq!(segment.points.len(), 113);
}

#[test]
fn gpx_reader_read_empty_name_tag() {
    let file = File::open("tests/fixtures/empty_name_tag.gpx").unwrap();
    let reader = BufReader::new(file);

    read(reader).unwrap();
}

#[test]
fn gpx_reader_read_test_with_track_numbers() {
    // Should not give an error, and should have all the correct data.
    let file = File::open("tests/fixtures/mousehole_to_paul.gpx").unwrap();
    let reader = BufReader::new(file);
    let result = read(reader);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.tracks.len(), 1);
    assert_eq!(result.tracks.first().unwrap().number, Some(1));
}

#[test]
fn gpx_reader_read_test_caltopo_export() -> Result<(), Box<dyn Error>> {
    let file = File::open("tests/fixtures/caltopo-export.gpx")?;
    let reader = BufReader::new(file);
    let res = read(reader)?;
    assert_eq!(res.tracks.len(), 2);

    // ensure day 1 tracks are parsed
    let track = &res.tracks[0];
    assert_eq!(track.name, Some("Day 01".to_string()));
    assert_eq!(track.segments.len(), 1);
    let segment = &track.segments[0];
    assert_eq!(segment.points.len(), 3);
    let point = &segment.points[0];
    assert_eq!(point.elevation, Some(3036.0));
    assert_eq!(
        point.point(),
        Point::new(-118.17100617103279, 36.44834803417325)
    );

    let expect = Some(
        PrimitiveDateTime::new(
            Date::from_calendar_date(2019, Month::August, 12).unwrap(),
            Time::from_hms(23, 45, 00).unwrap(),
        )
            .assume_utc()
            .into(),
    );

    assert_eq!(point.time, expect);

    // ensure day 2 tracks are parsed
    let track = &res.tracks[1];
    assert_eq!(track.name, Some("Day 02".to_string()));
    assert_eq!(track.segments.len(), 1);
    let segment = &track.segments[0];
    assert_eq!(segment.points.len(), 3);
    let point = &segment.points[2];
    assert_eq!(point.elevation, Some(2923.0));
    assert_eq!(
        point.point(),
        Point::new(-118.33698051050305, 36.49673483334482)
    );

    let expect = Some(
        PrimitiveDateTime::new(
            Date::from_calendar_date(2019, Month::August, 13).unwrap(),
            Time::from_hms(21, 46, 00).unwrap(),
        )
            .assume_utc()
            .into(),
    );

    assert_eq!(point.time, expect);

    Ok(())
}

#[test]
fn garmin_with_extensions() {
    // Should not give an error, and should have all the correct data.
    let file = File::open("tests/fixtures/garmin_with_extensions.gpx").unwrap();
    let reader = BufReader::new(file);

    let result = read(reader);
    assert!(result.is_ok());

    let result = result.unwrap();

    // Check the metadata, of course; here it has a time.
    let metadata = result.metadata.unwrap();

    let expect = PrimitiveDateTime::new(
        Date::from_calendar_date(2019, Month::May, 2).unwrap(),
        Time::from_hms(8, 53, 17).unwrap(),
    )
        .assume_utc()
        .into();

    assert_eq!(metadata.time.unwrap(), expect);

    assert_eq!(metadata.links.len(), 1);
    let link = &metadata.links[0];
    assert_eq!(link.href, "http://www.garmin.com");
    assert_eq!(link.text, Some(String::from("Garmin International")));

    // There should just be one track, "example gpx document".
    assert_eq!(result.tracks.len(), 1);
    let track = &result.tracks[0];

    assert_eq!(track.name, Some(String::from("2019-05-01 06:31:11 Tag")));

    // Each point has its own information; test elevation.
    assert_eq!(track.segments.len(), 2);
    let points = &track.segments[0].points;

    assert_eq!(points.len(), 35);
    assert_eq!(points[0].elevation, Some(860.0));
}

#[test]
fn viking_with_route_extensions() {
    // Should not give an error, and should have all the correct data.
    let file = File::open("tests/fixtures/viking_with_route_extensions.gpx").unwrap();
    let reader = BufReader::new(file);

    let result = read(reader);
    println!("result= {:#?}", result);
    assert!(result.is_ok());

    let result = result.unwrap();

    // There should just be one track, "example gpx document".
    assert_eq!(result.tracks.len(), 1);
    let track = &result.tracks[0];

    assert_eq!(track.name, Some(String::from("Trace")));

    // Each point has its own information; test elevation.
    assert_eq!(track.segments.len(), 1);
    let points = &track.segments[0].points;

    assert_eq!(points.len(), 5);
    assert_eq!(points[0].point().y(), 40.71631157206666);
}

#[test]
fn outdooractive_export() {
    // Should not give an error, and should have all the correct data.
    let file = File::open("tests/fixtures/outdooractive-export.gpx").unwrap();
    let reader = BufReader::new(file);

    let result = read(reader);
    assert!(result.is_ok());

    let result = result.unwrap();

    assert_eq!(result.tracks.len(), 1);
    let track = &result.tracks[0];

    assert_eq!(track.name, Some("Kilimanjaro - Machame Route".to_owned()));

    // Each point has its own information; test elevation.
    assert_eq!(track.segments.len(), 1);
    let points = &track.segments[0].points;

    assert_eq!(points.len(), 9);
    assert_eq!(points[0].point().y(), -3.173433);
}
