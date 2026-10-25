CREATE EXTENSION IF NOT EXISTS btree_gist;

CREATE TABLE IF NOT EXISTS tickets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users (id),
    schedule_id UUID NOT NULL REFERENCES train_schedules (id),
    seat_id UUID NOT NULL REFERENCES seats (id),
    travel_date DATE NOT NULL,
    from_station_id UUID NOT NULL REFERENCES stations (id),
    to_station_id UUID NOT NULL REFERENCES stations (id),
    -- Stretch of the line the seat is occupied for, as station positions [lower, upper).
    -- 2 -> 5 is [2,5) and 5 -> 9 is [5,9): they don't overlap, so the seat can be reused at station 5.
    segment INT4RANGE NOT NULL,
    status TEXT NOT NULL DEFAULT 'booked' CHECK (status IN ('booked', 'cancelled')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (from_station_id <> to_station_id),
    -- No two active bookings for the same seat, train and date may cover overlapping stretches.
    CONSTRAINT tickets_no_overlapping_seat EXCLUDE USING gist (
        seat_id WITH =,
        schedule_id WITH =,
        travel_date WITH =,
        segment WITH &&
    ) WHERE (status = 'booked')
);

CREATE INDEX IF NOT EXISTS tickets_user_id_idx ON tickets (user_id);
