use chrono::NaiveDate;
use sqlx::{Error as SqlxError, PgPool, query_as};
use uuid::Uuid;

use crate::models::station::{CoachAvailability, Seat, Station, TripOption};

pub async fn fetch_train_stations(db: &PgPool) -> Result<Vec<Station>, SqlxError> {
    query_as::<_, Station>(
        r#"
    SELECT id, name, position
    FROM stations
    ORDER BY position ASC
    "#,
    )
    .fetch_all(db)
    .await
}

pub async fn fetch_station_by_id(db: &PgPool, id: Uuid) -> Result<Option<Station>, SqlxError> {
    sqlx::query_as::<_, Station>(
        r#"
        SELECT id, name, position
        FROM stations
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(db)
    .await
}

pub async fn fetch_coaches(
    db: &PgPool,
    schedule_id: Uuid,
    from_pos: i32,
    to_pos: i32,
    travel_date: NaiveDate,
    class_code: &str,
) -> Result<Vec<CoachAvailability>, SqlxError> {
    sqlx::query_as::<_, CoachAvailability>(
        r#"
    SELECT
        co.coach_number,
        cl.code AS class_code,
        cl.name AS class_name,
        COUNT(s.id) AS total_seats,
        COUNT(s.id) FILTER (WHERE NOT EXISTS (
            SELECT 1
            FROM tickets t
            WHERE t.seat_id = s.id
              AND t.schedule_id = $1
              AND t.travel_date = $2
              AND t.status = 'booked'
              AND t.segment && int4range(LEAST($3, $4), GREATEST($3, $4))
        )) AS available_seats
    FROM coaches co
    JOIN seat_classes cl ON cl.code = co.class_code
    JOIN seats s ON s.coach_id = co.id
    WHERE co.class_code = $5
    GROUP BY co.coach_number, cl.code, cl.name
    ORDER BY co.coach_number
    "#,
    )
    .bind(schedule_id)
    .bind(travel_date)
    .bind(from_pos)
    .bind(to_pos)
    .bind(class_code)
    .fetch_all(db)
    .await
}

pub async fn fetch_train_for_trip(
    db: &PgPool,
    from_pos: i32,
    to_pos: i32,
    travel_date: NaiveDate,
) -> Result<Vec<TripOption>, SqlxError> {
    sqlx::query_as::<_, TripOption>(
        r#"
    SELECT
        ts.id AS schedule_id,
        ts.direction,
        ts.departure_time,
        ts.departure_time + make_interval(
            mins => ts.minutes_between_stations *
                CASE ts.direction
                    WHEN 'forward' THEN $1 - 1
                    ELSE (SELECT MAX(position) FROM stations) - $1
                END
        ) AS boarding_time,
        ts.departure_time + make_interval(
            mins => ts.minutes_between_stations *
                CASE ts.direction
                    WHEN 'forward' THEN $2 - 1
                    ELSE (SELECT MAX(position) FROM stations) - $2
                END
        ) AS arrival_time,
        availability.classes
    FROM train_schedules ts
    CROSS JOIN LATERAL (
        SELECT json_agg(
            json_build_object(
                'class_code', counts.class_code,
                'class_name', counts.class_name,
                'total_seats', counts.total_seats,
                'reserved_seats', counts.reserved_seats,
                'available_seats', counts.total_seats - counts.reserved_seats
            )
            ORDER BY counts.rank
        ) AS classes
        FROM (
            SELECT
                cl.code AS class_code,
                cl.name AS class_name,
                cl.rank,
                COUNT(s.id) AS total_seats,
                COUNT(s.id) FILTER (WHERE EXISTS (
                    SELECT 1
                    FROM tickets t
                    WHERE t.seat_id = s.id
                      AND t.schedule_id = ts.id
                      AND t.travel_date = $3
                      AND t.status = 'booked'
                      AND t.segment && int4range(LEAST($1, $2), GREATEST($1, $2))
                )) AS reserved_seats
            FROM seats s
            JOIN coaches co ON co.id = s.coach_id
            JOIN seat_classes cl ON cl.code = co.class_code
            GROUP BY cl.code, cl.name, cl.rank
        ) counts
    ) availability
    WHERE ts.direction = CASE WHEN $2 > $1 THEN 'forward' ELSE 'backward' END
    ORDER BY ts.departure_time
    "#,
    )
    .bind(from_pos)
    .bind(to_pos)
    .bind(travel_date)
    .fetch_all(db)
    .await
}

pub async fn fetch_seats_for_coach(
    db: &PgPool,
    schedule_id: Uuid,
    from_pos: i32,
    to_pos: i32,
    travel_date: NaiveDate,
    coach_number: i32,
) -> Result<Vec<Seat>, SqlxError> {
    sqlx::query_as::<_, Seat>(
        r#"
    SELECT
        s.id,
        co.coach_number,
        s.seat_number,
        s.seat_type,
        s.side,
        EXISTS (
            SELECT 1
            FROM tickets t
            WHERE t.seat_id = s.id
              AND t.schedule_id = $1
              AND t.travel_date = $2
              AND t.status = 'booked'
              AND t.segment && int4range(LEAST($3, $4), GREATEST($3, $4))
        ) AS is_taken
    FROM seats s
    JOIN coaches co ON co.id = s.coach_id
    WHERE co.coach_number = $5
    ORDER BY s.seat_number
    "#,
    )
    .bind(schedule_id)
    .bind(travel_date)
    .bind(from_pos)
    .bind(to_pos)
    .bind(coach_number)
    .fetch_all(db)
    .await
}
