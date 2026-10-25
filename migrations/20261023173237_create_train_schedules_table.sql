CREATE TABLE IF NOT EXISTS train_schedules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    direction TEXT NOT NULL CHECK (direction IN ('forward', 'backward')),
    departure_time TIME NOT NULL,
    minutes_between_stations INTEGER NOT NULL CHECK (minutes_between_stations > 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (direction, departure_time)
);

INSERT INTO train_schedules (direction, departure_time, minutes_between_stations) VALUES
    ('forward', '08:00', 20),
    ('backward', '08:00', 20),
    ('forward', '16:00', 20),
    ('backward', '16:00', 20)
ON CONFLICT (direction, departure_time) DO NOTHING;
