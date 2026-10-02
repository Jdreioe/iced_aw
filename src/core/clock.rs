//! Helper functions for calculating the clock

use iced_core::Point;
use std::fmt::Display;

/// The size of the period on the clock based on the clock's size.
pub const PERIOD_PERCENTAGE: f32 = 0.1;

/// The radius of the hour on the clock based on the clock's size.
pub const HOUR_RADIUS_PERCENTAGE: f32 = 0.4;

/// The radius of the minute on the clock based on the clock's size.
pub const MINUTE_RADIUS_PERCENTAGE: f32 = 0.65;

/// The radius of the second on the clock based on the clock's size.
pub const SECOND_RADIUS_PERCENTAGE: f32 = 0.9;

/// The radius of the hour without the seconds on the clock based on the
/// clock's size.
pub const HOUR_RADIUS_PERCENTAGE_NO_SECONDS: f32 = 0.5;

/// The radius of the minute without the seconds on the clock based on the
/// clock's size.
pub const MINUTE_RADIUS_PERCENTAGE_NO_SECONDS: f32 = 0.9;

/// The current period of the clock.
#[derive(Clone, Debug)]
pub enum Period {
    /// Ante meridiem: Before noon.
    AM,
    /// Post meridiem: After noon.
    PM,
}

impl Display for Period {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::AM => "AM",
                Self::PM => "PM",
            }
        )
    }
}

/// The radius nearest to the cursor position.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NearestRadius {
    /// Nothing is near to the cursor position.
    None,
    /// Period is the nearest radius to the cursor position.
    Period,
    /// Hour is the nearest radius to the cursor position.
    Hour,
    /// The inner ring of hours, 12 to 23, of a 24 hour clock is the nearest
    /// radius to the cursor position.
    InnerHour,
    /// Minute is the nearest radius to the cursor position.
    Minute,
    /// Second is the nearest radius to the cursor position.
    Second,
}

/// The radii of a 24 hour clock: the hours 0 to 11 on one ring and 12 to 23 on
/// a ring inside it, instead of a period to switch between them.
const HOUR_RADIUS_PERCENTAGE_24H: f32 = 0.64;
const INNER_HOUR_RADIUS_PERCENTAGE_24H: f32 = 0.4;
const HOUR_RADIUS_PERCENTAGE_24H_SECONDS: f32 = 0.48;
const INNER_HOUR_RADIUS_PERCENTAGE_24H_SECONDS: f32 = 0.3;
const MINUTE_RADIUS_PERCENTAGE_24H_SECONDS: f32 = 0.69;

/// The rings of a clock with the given radius, by what it shows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Radii {
    /// The period in the center, on a 12 hour clock only.
    pub period: Option<f32>,
    /// The hours, 0 to 11 on a 24 hour clock.
    pub hour: f32,
    /// The hours 12 to 23, on a 24 hour clock only.
    pub inner_hour: Option<f32>,
    /// The minutes.
    pub minute: f32,
    /// The seconds, when they are shown.
    pub second: Option<f32>,
}

impl Radii {
    /// The rings of a clock with the given radius.
    #[must_use]
    pub fn new(radius: f32, use_24h: bool, show_seconds: bool) -> Self {
        let (period, hour, inner_hour, minute, second) = match (use_24h, show_seconds) {
            (false, false) => (
                Some(PERIOD_PERCENTAGE),
                HOUR_RADIUS_PERCENTAGE_NO_SECONDS,
                None,
                MINUTE_RADIUS_PERCENTAGE_NO_SECONDS,
                None,
            ),
            (false, true) => (
                Some(PERIOD_PERCENTAGE),
                HOUR_RADIUS_PERCENTAGE,
                None,
                MINUTE_RADIUS_PERCENTAGE,
                Some(SECOND_RADIUS_PERCENTAGE),
            ),
            (true, false) => (
                None,
                HOUR_RADIUS_PERCENTAGE_24H,
                Some(INNER_HOUR_RADIUS_PERCENTAGE_24H),
                MINUTE_RADIUS_PERCENTAGE_NO_SECONDS,
                None,
            ),
            (true, true) => (
                None,
                HOUR_RADIUS_PERCENTAGE_24H_SECONDS,
                Some(INNER_HOUR_RADIUS_PERCENTAGE_24H_SECONDS),
                MINUTE_RADIUS_PERCENTAGE_24H_SECONDS,
                Some(SECOND_RADIUS_PERCENTAGE),
            ),
        };
        Self {
            period: period.map(|p| p * radius),
            hour: hour * radius,
            inner_hour: inner_hour.map(|p| p * radius),
            minute: minute * radius,
            second: second.map(|p| p * radius),
        }
    }

    /// The ring nearest to the cursor position.
    #[must_use]
    pub fn nearest(&self, cursor_position: Point, center: Point) -> NearestRadius {
        let mut radii = vec![
            (self.hour, NearestRadius::Hour),
            (self.minute, NearestRadius::Minute),
        ];
        if let Some(period) = self.period {
            radii.push((period, NearestRadius::Period));
        }
        if let Some(inner_hour) = self.inner_hour {
            radii.push((inner_hour, NearestRadius::InnerHour));
        }
        if let Some(second) = self.second {
            radii.push((second, NearestRadius::Second));
        }
        nearest_radius(&radii, cursor_position, center)
    }

    /// Whether the cursor position is nearer the inner ring of hours than the
    /// outer one. Always false on a 12 hour clock.
    #[must_use]
    pub fn on_inner_hour(&self, cursor_position: Point, center: Point) -> bool {
        self.inner_hour.is_some_and(|inner_hour| {
            let distance = cursor_position.distance(center);
            (inner_hour - distance).abs() < (self.hour - distance).abs()
        })
    }
}

/// Distributes the amount of points on a circle with the given radius around the
/// center.
#[must_use]
pub fn circle_points(distance_radius: f32, center: Point, amount: u16) -> Vec<Point> {
    let part = std::f32::consts::TAU / f32::from(amount);

    let rotation =
        |(x, y): (f32, f32), t: f32| (x * t.cos() - y * t.sin(), x * t.sin() + y * t.cos());

    let points: Vec<(f32, f32)> = (0..amount).fold(Vec::new(), |mut v, i| {
        v.push(rotation((0.0, -distance_radius), part * f32::from(i)));
        v
    });

    let points: Vec<Point> = points
        .iter()
        .map(|p| Point::new(p.0 + center.x, p.1 + center.y))
        .collect();

    points
}

/// # Panics
/// Determines the nearest point with the smallest distance to the cursor
/// position. The index of the point is returned.
/// Will panic if distance vec can not compare a and b
#[must_use]
pub fn nearest_point(points: &[Point], cursor_position: Point) -> usize {
    let mut distance_vec: Vec<(usize, f32)> = points
        .iter()
        .enumerate()
        .map(|(i, p)| (i, p.distance(cursor_position)))
        .collect();

    distance_vec.sort_by(|a, b| a.1.partial_cmp(&b.1).expect("Should be comparable"));

    distance_vec[0].0
}

/// # Panics
/// Determining the nearest radius to the position of the cursor position based
/// on the distance to the center.
/// Will panic if distance vec can not compare a and b
#[must_use]
pub fn nearest_radius(
    radii: &[(f32, NearestRadius)],
    cursor_position: Point,
    center: Point,
) -> NearestRadius {
    let distance = cursor_position.distance(center);

    let mut distance_vec: Vec<(f32, &NearestRadius)> = radii
        .iter()
        .map(|(r, n)| ((r - distance).abs(), n))
        .collect();

    distance_vec.sort_by(|a, b| a.0.partial_cmp(&b.0).expect("Should be comparable"));

    distance_vec[0].1.clone()
}

#[cfg(test)]
mod tests {
    use iced_core::{Point, Vector};

    use super::{NearestRadius, circle_points, nearest_point, nearest_radius};

    #[test]
    fn circle_points_test() {
        let points = circle_points(10.0, Point::new(0.0, 0.0), 10);
        let expected = vec![
            Point { x: 0.0, y: -10.0 },
            Point {
                x: 5.877_852_4,
                y: -8.09017,
            },
            Point {
                x: 9.510_566,
                y: -3.090_169_7,
            },
            Point {
                x: 9.510_565,
                y: 3.090_170_4,
            },
            Point {
                x: 5.877_852,
                y: 8.090_171,
            },
            Point {
                x: -0.000_000_874_227_8,
                y: 10.0,
            },
            Point {
                x: -5.877_853_4,
                y: 8.09017,
            },
            Point {
                x: -9.510_565,
                y: 3.090_170_9,
            },
            Point {
                x: -9.510_565,
                y: -3.090_171_3,
            },
            Point {
                x: -5.877_849_6,
                y: -8.090_173,
            },
        ];

        assert_eq!(points, expected);
    }

    #[test]
    fn nearest_radius_test() {
        let radii = vec![
            (10.0, NearestRadius::Period),
            (20.0, NearestRadius::Hour),
            (30.0, NearestRadius::Minute),
            (40.0, NearestRadius::Second),
        ];

        let center = Point::new(0.0, 0.0);

        let cursor_position = Point::new(5.0, 0.0);
        let result = nearest_radius(&radii, cursor_position, center);
        assert_eq!(result, NearestRadius::Period);

        let cursor_position = Point::new(16.0, 0.0);
        let result = nearest_radius(&radii, cursor_position, center);
        assert_eq!(result, NearestRadius::Hour);

        let cursor_position = Point::new(26.0, 0.0);
        let result = nearest_radius(&radii, cursor_position, center);
        assert_eq!(result, NearestRadius::Minute);

        let cursor_position = Point::new(36.0, 0.0);
        let result = nearest_radius(&radii, cursor_position, center);
        assert_eq!(result, NearestRadius::Second);
    }

    #[test]
    fn nearest_point_test() {
        let points = circle_points(10.0, Point::new(0.0, 0.0), 10);

        let mut index = 5;
        let cursor_position = points[index] + Vector::new(1.0, -1.0);
        let mut result = nearest_point(&points, cursor_position);
        assert_eq!(index, result);

        index = 0;
        let cursor_position = points[index] + Vector::new(-1.0, 1.0);
        result = nearest_point(&points, cursor_position);
        assert_eq!(index, result);
    }

    #[test]
    fn period_display_am() {
        use super::Period;
        let period = Period::AM;
        assert_eq!(format!("{period}"), "AM");
    }

    #[test]
    fn period_display_pm() {
        use super::Period;
        let period = Period::PM;
        assert_eq!(format!("{period}"), "PM");
    }

    #[test]
    fn circle_points_with_offset_center() {
        let points = circle_points(5.0, Point::new(10.0, 20.0), 4);
        assert_eq!(points.len(), 4);
        // Top point should be at (10, 15) - center.y - radius
        assert!((points[0].x - 10.0).abs() < 0.001);
        assert!((points[0].y - 15.0).abs() < 0.001);
    }

    #[test]
    fn circle_points_single_point() {
        let points = circle_points(10.0, Point::new(0.0, 0.0), 1);
        assert_eq!(points.len(), 1);
        // Single point should be at top (0, -10)
        assert!((points[0].x - 0.0).abs() < 0.001);
        assert!((points[0].y - (-10.0)).abs() < 0.001);
    }

    #[test]
    fn circle_points_twelve_points() {
        let points = circle_points(10.0, Point::new(0.0, 0.0), 12);
        assert_eq!(points.len(), 12);
        // 12 points should be evenly distributed (like clock hours)
        // First point should be at top
        assert!((points[0].x - 0.0).abs() < 0.001);
        assert!((points[0].y - (-10.0)).abs() < 0.001);
    }

    #[test]
    fn nearest_point_single_point() {
        let points = vec![Point::new(5.0, 5.0)];
        let cursor = Point::new(100.0, 100.0);
        let result = nearest_point(&points, cursor);
        assert_eq!(result, 0);
    }

    #[test]
    fn nearest_point_exact_match() {
        let points = circle_points(10.0, Point::new(0.0, 0.0), 8);
        // Cursor exactly on point 3
        let cursor = points[3];
        let result = nearest_point(&points, cursor);
        assert_eq!(result, 3);
    }

    #[test]
    fn nearest_radius_at_exact_radius() {
        let radii = vec![
            (10.0, NearestRadius::Period),
            (20.0, NearestRadius::Hour),
            (30.0, NearestRadius::Minute),
        ];
        let center = Point::new(0.0, 0.0);
        // Cursor exactly at radius 20
        let cursor = Point::new(20.0, 0.0);
        let result = nearest_radius(&radii, cursor, center);
        assert_eq!(result, NearestRadius::Hour);
    }

    #[test]
    fn nearest_radius_at_center() {
        let radii = vec![
            (10.0, NearestRadius::Period),
            (20.0, NearestRadius::Hour),
            (30.0, NearestRadius::Minute),
        ];
        let center = Point::new(0.0, 0.0);
        // Cursor at center
        let cursor = Point::new(0.0, 0.0);
        let result = nearest_radius(&radii, cursor, center);
        // Should pick the smallest radius (Period at 10.0)
        assert_eq!(result, NearestRadius::Period);
    }

    #[test]
    fn nearest_radius_beyond_largest() {
        let radii = vec![
            (10.0, NearestRadius::Period),
            (20.0, NearestRadius::Hour),
            (30.0, NearestRadius::Minute),
        ];
        let center = Point::new(0.0, 0.0);
        // Cursor far beyond all radii
        let cursor = Point::new(100.0, 0.0);
        let result = nearest_radius(&radii, cursor, center);
        // Should still pick the closest (Minute at 30.0)
        assert_eq!(result, NearestRadius::Minute);
    }

    #[test]
    fn nearest_radius_with_offset_center() {
        let radii = vec![(5.0, NearestRadius::Period), (15.0, NearestRadius::Hour)];
        let center = Point::new(50.0, 50.0);
        let cursor = Point::new(62.0, 50.0);
        // Distance from center is 12, closer to Hour (15) than Period (5)
        // |5 - 12| = 7, |15 - 12| = 3, so Hour is closer
        let result = nearest_radius(&radii, cursor, center);
        assert_eq!(result, NearestRadius::Hour);
    }

    #[test]
    fn nearest_radius_none_variant() {
        let radii = vec![(10.0, NearestRadius::None), (20.0, NearestRadius::Hour)];
        let center = Point::new(0.0, 0.0);
        let cursor = Point::new(11.0, 0.0);
        let result = nearest_radius(&radii, cursor, center);
        // Should pick None as it's closer to radius 10
        assert_eq!(result, NearestRadius::None);
    }

    #[test]
    fn circle_points_large_amount() {
        let points = circle_points(10.0, Point::new(0.0, 0.0), 60);
        assert_eq!(points.len(), 60);
        // All points should be approximately 10.0 away from center
        let center = Point::new(0.0, 0.0);
        for point in points {
            let distance = point.distance(center);
            assert!((distance - 10.0).abs() < 0.001);
        }
    }

    #[test]
    fn a_24_hour_clock_has_an_inner_ring_of_hours_and_no_period() {
        use super::Radii;
        let center = Point::new(100.0, 100.0);
        let radii = Radii::new(100.0, true, false);
        assert_eq!(radii.period, None);
        assert_eq!(
            radii.nearest(Point::new(100.0, 60.0), center),
            NearestRadius::InnerHour
        );
        assert_eq!(
            radii.nearest(Point::new(100.0, 36.0), center),
            NearestRadius::Hour
        );
        assert_eq!(
            radii.nearest(Point::new(100.0, 10.0), center),
            NearestRadius::Minute
        );
        // The center is nearest the inner hours, not a period.
        assert_eq!(radii.nearest(center, center), NearestRadius::InnerHour);
        assert!(radii.on_inner_hour(Point::new(100.0, 60.0), center));
        assert!(!radii.on_inner_hour(Point::new(100.0, 36.0), center));

        let radii = Radii::new(100.0, false, false);
        assert_eq!(radii.inner_hour, None);
        assert_eq!(radii.nearest(center, center), NearestRadius::Period);
        assert!(!radii.on_inner_hour(Point::new(100.0, 60.0), center));
    }

    #[test]
    fn constants_are_defined() {
        use super::*;
        // Just verify the constants are accessible and have expected values

        assert_eq!(PERIOD_PERCENTAGE, 0.1);
        assert_eq!(HOUR_RADIUS_PERCENTAGE, 0.4);
        assert_eq!(MINUTE_RADIUS_PERCENTAGE, 0.65);
        assert_eq!(SECOND_RADIUS_PERCENTAGE, 0.9);
        assert_eq!(HOUR_RADIUS_PERCENTAGE_NO_SECONDS, 0.5);
        assert_eq!(MINUTE_RADIUS_PERCENTAGE_NO_SECONDS, 0.9);
    }
}
