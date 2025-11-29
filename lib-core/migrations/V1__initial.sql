CREATE TABLE entities (
    id INTEGER PRIMARY KEY
);

CREATE TABLE map_info (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    width INTEGER NOT NULL,
    height INTEGER NOT NULL,
    name TEXT NOT NULL
);

CREATE TABLE walls (
    id INTEGER PRIMARY KEY,
    FOREIGN KEY(id) REFERENCES entities(id)
);

CREATE TABLE grid_positions (
    wall_id INTEGER,
    x INTEGER,
    y INTEGER,
    FOREIGN KEY(wall_id) REFERENCES walls(id)
);
