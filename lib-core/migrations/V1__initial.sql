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

CREATE TABLE grid_coords (
    entity_id INTEGER,
    x INTEGER,
    y INTEGER,
    FOREIGN KEY(entity_id) REFERENCES entities(id)
);

CREATE TABLE main_bases (
    id INTEGER PRIMARY KEY,
    FOREIGN KEY(id) REFERENCES entities(id)
);

CREATE TABLE exploration_centers (
    id INTEGER PRIMARY KEY,
    FOREIGN KEY(id) REFERENCES entities(id)
);

CREATE TABLE energy_relays (
    id INTEGER PRIMARY KEY,
    FOREIGN KEY(id) REFERENCES entities(id)
);

CREATE TABLE mining_complexes (
    id INTEGER PRIMARY KEY,
    FOREIGN KEY(id) REFERENCES entities(id)
);

CREATE TABLE dark_ores (
    id INTEGER PRIMARY KEY,
    amount INTEGER NOT NULL,
    FOREIGN KEY(id) REFERENCES entities(id)
);

CREATE TABLE healths (
    entity_id INTEGER PRIMARY KEY,
    current REAL NOT NULL,
    FOREIGN KEY(entity_id) REFERENCES entities(id)
);
