CREATE TABLE IF NOT EXISTS seat_classes (
    code TEXT PRIMARY KEY CHECK (code IN ('first', 'business', 'standard')),
    name TEXT NOT NULL UNIQUE,
    rank INTEGER NOT NULL UNIQUE
);

INSERT INTO seat_classes (code, name, rank) VALUES
    ('first', 'Monarch', 1),
    ('business', 'Horizon', 2),
    ('standard', 'Voyager', 3)
ON CONFLICT (code) DO NOTHING;

-- One coach = one carriage of the train.
CREATE TABLE IF NOT EXISTS coaches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    coach_number INTEGER NOT NULL UNIQUE CHECK (coach_number > 0),
    class_code TEXT NOT NULL REFERENCES seat_classes (code),
    seat_count INTEGER NOT NULL CHECK (seat_count > 0)
);

INSERT INTO coaches (coach_number, class_code, seat_count)
SELECT 1, 'first', 10
UNION ALL
SELECT n, 'business', 40 FROM generate_series(2, 3) AS n
UNION ALL
SELECT n, 'standard', 80 FROM generate_series(4, 10) AS n
ON CONFLICT (coach_number) DO NOTHING;

-- Seat numbers restart in every coach.
CREATE TABLE IF NOT EXISTS seats (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    coach_id UUID NOT NULL REFERENCES coaches (id),
    seat_number INTEGER NOT NULL CHECK (seat_number > 0),
    seat_type TEXT NOT NULL CHECK (seat_type IN ('window', 'middle', 'aisle')),
    side TEXT NOT NULL CHECK (side IN ('left', 'right')),
    UNIQUE (coach_id, seat_number)
);

-- Each class repeats a fixed row pattern inside every coach:
--   first:    window-left, window-right
--   business: window-left, aisle-left, aisle-right, window-right
--   standard: window-left, middle-left, aisle-left, aisle-right, middle-right, window-right
WITH layouts (class_code, seat_types, sides) AS (
    VALUES
        ('first',
            ARRAY['window', 'window'],
            ARRAY['left', 'right']),
        ('business',
            ARRAY['window', 'aisle', 'aisle', 'window'],
            ARRAY['left', 'left', 'right', 'right']),
        ('standard',
            ARRAY['window', 'middle', 'aisle', 'aisle', 'middle', 'window'],
            ARRAY['left', 'left', 'left', 'right', 'right', 'right'])
)
INSERT INTO seats (coach_id, seat_number, seat_type, side)
SELECT
    c.id,
    n,
    l.seat_types[((n - 1) % array_length(l.seat_types, 1)) + 1],
    l.sides[((n - 1) % array_length(l.sides, 1)) + 1]
FROM coaches c
JOIN layouts l ON l.class_code = c.class_code
CROSS JOIN LATERAL generate_series(1, c.seat_count) AS n
ON CONFLICT (coach_id, seat_number) DO NOTHING;
