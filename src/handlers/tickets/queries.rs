use chrono::NaiveDate;
use chrono_tz::Tz;
use sqlx::{PgPool, error::Error as SqlxError};
use uuid::Uuid;

use crate::{
    models::tickets::{BookedTicket, UserTicket},
    utils::get_current_date_time,
};

pub async fn insert_book_ticket(
    db: &PgPool,
    user_id: Uuid,
    from: Uuid,
    to: Uuid,
    schedule_id: Uuid,
    seat_id: Uuid,
    date: NaiveDate,
) -> Result<BookedTicket, SqlxError> {
    sqlx::query_as::<_, BookedTicket>(
        r#"
    WITH new_ticket AS (
        INSERT INTO tickets (user_id, schedule_id, seat_id, travel_date,
                             from_station_id, to_station_id, segment)
        SELECT $1, $2, $3, $4, f.id, t.id,
               int4range(LEAST(f.position, t.position), GREATEST(f.position, t.position))
        FROM stations f, stations t
        WHERE f.id = $5 AND t.id = $6
        RETURNING id, schedule_id, seat_id, travel_date, from_station_id, to_station_id,
                  status, created_at
    )
    SELECT
        nt.id,
        nt.schedule_id,
        nt.travel_date,
        nt.status,
        nt.created_at,
        f.id AS from_station_id,
        f.name AS from_station_name,
        t.id AS to_station_id,
        t.name AS to_station_name,
        co.coach_number,
        cl.code AS class_code,
        cl.name AS class_name,
        s.id AS seat_id,
        s.seat_number,
        s.seat_type,
        s.side
    FROM new_ticket nt
    JOIN stations f ON f.id = nt.from_station_id
    JOIN stations t ON t.id = nt.to_station_id
    JOIN seats s ON s.id = nt.seat_id
    JOIN coaches co ON co.id = s.coach_id
    JOIN seat_classes cl ON cl.code = co.class_code
    "#,
    )
    .bind(user_id)
    .bind(schedule_id)
    .bind(seat_id)
    .bind(date)
    .bind(from)
    .bind(to)
    .fetch_one(db)
    .await
}

pub async fn fetch_user_tickets(
    db: &PgPool,
    user_id: Uuid,
    all: bool,
    tz: &Tz,
    limit: i64,
    offset: i64,
) -> Result<Vec<UserTicket>, SqlxError> {
    sqlx::query_as::<_, UserTicket>(
        r#"
    SELECT *
    FROM (
        SELECT
            t.id,
            t.status,
            t.travel_date,
            t.created_at,
            ts.id AS schedule_id,
            ts.direction,
            ts.departure_time,
            ts.departure_time + make_interval(
                mins => ts.minutes_between_stations *
                    CASE ts.direction
                        WHEN 'forward' THEN f.position - 1
                        ELSE (SELECT MAX(position) FROM stations) - f.position
                    END
            ) AS boarding_time,
            ts.departure_time + make_interval(
                mins => ts.minutes_between_stations *
                    CASE ts.direction
                        WHEN 'forward' THEN d.position - 1
                        ELSE (SELECT MAX(position) FROM stations) - d.position
                    END
            ) AS arrival_time,
            f.id AS from_station_id,
            f.name AS from_station_name,
            d.id AS to_station_id,
            d.name AS to_station_name,
            co.coach_number,
            cl.code AS class_code,
            cl.name AS class_name,
            s.id AS seat_id,
            s.seat_number,
            s.seat_type,
            s.side
        FROM tickets t
        JOIN train_schedules ts ON ts.id = t.schedule_id
        JOIN stations f ON f.id = t.from_station_id
        JOIN stations d ON d.id = t.to_station_id
        JOIN seats s ON s.id = t.seat_id
        JOIN coaches co ON co.id = s.coach_id
        JOIN seat_classes cl ON cl.code = co.class_code
        WHERE t.user_id = $1
    ) user_tickets
    WHERE $2 OR (travel_date + arrival_time) > $3
    ORDER BY travel_date DESC, departure_time DESC, created_at DESC
    LIMIT $4 OFFSET $5
    "#,
    )
    .bind(user_id)
    .bind(all)
    .bind(get_current_date_time(tz))
    .bind(limit)
    .bind(offset)
    .fetch_all(db)
    .await
}

pub async fn count_user_tickets(
    db: &PgPool,
    user_id: Uuid,
    all: bool,
    tz: &Tz,
) -> Result<i64, SqlxError> {
    sqlx::query_scalar::<_, i64>(
        r#"
    SELECT COUNT(*)
    FROM tickets t
    JOIN train_schedules ts ON ts.id = t.schedule_id
    JOIN stations d ON d.id = t.to_station_id
    WHERE t.user_id = $1
      AND ($2 OR (t.travel_date + ts.departure_time + make_interval(
            mins => ts.minutes_between_stations *
                CASE ts.direction
                    WHEN 'forward' THEN d.position - 1
                    ELSE (SELECT MAX(position) FROM stations) - d.position
                END
          )) > $3)
    "#,
    )
    .bind(user_id)
    .bind(all)
    .bind(get_current_date_time(tz))
    .fetch_one(db)
    .await
}

pub async fn fetch_single_ticket(
    db: &PgPool,
    ticket_id: Uuid,
    user_id: Uuid,
) -> Result<Option<UserTicket>, SqlxError> {
    sqlx::query_as::<_, UserTicket>(
        r#"
    SELECT
        t.id,
        t.status,
        t.travel_date,
        t.created_at,
        ts.id AS schedule_id,
        ts.direction,
        ts.departure_time,
        ts.departure_time + make_interval(
            mins => ts.minutes_between_stations *
                CASE ts.direction
                    WHEN 'forward' THEN f.position - 1
                    ELSE (SELECT MAX(position) FROM stations) - f.position
                END
        ) AS boarding_time,
        ts.departure_time + make_interval(
            mins => ts.minutes_between_stations *
                CASE ts.direction
                    WHEN 'forward' THEN d.position - 1
                    ELSE (SELECT MAX(position) FROM stations) - d.position
                END
        ) AS arrival_time,
        f.id AS from_station_id,
        f.name AS from_station_name,
        d.id AS to_station_id,
        d.name AS to_station_name,
        co.coach_number,
        cl.code AS class_code,
        cl.name AS class_name,
        s.id AS seat_id,
        s.seat_number,
        s.seat_type,
        s.side
    FROM tickets t
    JOIN train_schedules ts ON ts.id = t.schedule_id
    JOIN stations f ON f.id = t.from_station_id
    JOIN stations d ON d.id = t.to_station_id
    JOIN seats s ON s.id = t.seat_id
    JOIN coaches co ON co.id = s.coach_id
    JOIN seat_classes cl ON cl.code = co.class_code
    WHERE t.id = $1 AND t.user_id = $2
    "#,
    )
    .bind(ticket_id)
    .bind(user_id)
    .fetch_optional(db)
    .await
}
