CREATE TABLE IF NOT EXISTS stations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    position INTEGER NOT NULL UNIQUE CHECK (position > 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO stations (name, position) VALUES
    ('Neon Harbor', 1),
    ('Copperfield Junction', 2),
    ('Starlight Terminal', 3),
    ('Ironwood Crossing', 4),
    ('Velvet Hollow', 5),
    ('Thunder Ridge', 6),
    ('Crystal Falls', 7),
    ('Ember Valley', 8),
    ('Silvermoon Bay', 9),
    ('Aurora Summit', 10)
ON CONFLICT (name) DO NOTHING;
