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

CREATE TABLE grid_imprints (
    id INTEGER PRIMARY KEY,
    shape TEXT NOT NULL,
    width INTEGER,
    height INTEGER,
    FOREIGN KEY(id) REFERENCES entities(id)
);

CREATE TABLE quantum_fields (
    id INTEGER PRIMARY KEY,
    current_layer INTEGER NOT NULL,
    current_layer_progress INTEGER NOT NULL,
    is_expedition_target BOOLEAN NOT NULL,
    FOREIGN KEY(id) REFERENCES entities(id)
);

CREATE TABLE healths (
    entity_id INTEGER PRIMARY KEY,
    current REAL NOT NULL,
    FOREIGN KEY(entity_id) REFERENCES entities(id)
);

CREATE TABLE world_positions (
    entity_id INTEGER PRIMARY KEY,
    x REAL NOT NULL,
    y REAL NOT NULL,
    FOREIGN KEY(entity_id) REFERENCES entities(id)
);

CREATE TABLE disabled_by_player (
    entity_id INTEGER PRIMARY KEY,
    FOREIGN KEY(entity_id) REFERENCES entities(id)
);

CREATE TABLE expedition_drones (
    id INTEGER PRIMARY KEY,
    target_id INTEGER NOT NULL,
    FOREIGN KEY(id) REFERENCES entities(id)
);

CREATE TABLE cannonballs (
    id INTEGER PRIMARY KEY,
    target_x REAL NOT NULL,
    target_y REAL NOT NULL,
    damage REAL NOT NULL,
    initial_distance REAL NOT NULL,
    FOREIGN KEY(id) REFERENCES entities(id)
);

CREATE TABLE laser_darts (
    id INTEGER PRIMARY KEY,
    target_wisp_id INTEGER,
    vector_x REAL NOT NULL,
    vector_y REAL NOT NULL,
    damage REAL NOT NULL,
    FOREIGN KEY(id) REFERENCES entities(id)
);

CREATE TABLE rockets (
    id INTEGER PRIMARY KEY,
    target_wisp_id INTEGER,
    rotation_z REAL NOT NULL,
    damage REAL NOT NULL,
    FOREIGN KEY(id) REFERENCES entities(id)
);

CREATE TABLE tower_cannons (
    id INTEGER PRIMARY KEY,
    FOREIGN KEY(id) REFERENCES entities(id)
);

CREATE TABLE tower_blasters (
    id INTEGER PRIMARY KEY,
    FOREIGN KEY(id) REFERENCES entities(id)
);

CREATE TABLE tower_rocket_launchers (
    id INTEGER PRIMARY KEY,
    FOREIGN KEY(id) REFERENCES entities(id)
);

CREATE TABLE tower_emitters (
    id INTEGER PRIMARY KEY,
    FOREIGN KEY(id) REFERENCES entities(id)
);

CREATE TABLE ripples (
    id INTEGER PRIMARY KEY,
    max_radius REAL NOT NULL,
    current_radius REAL NOT NULL,
    FOREIGN KEY(id) REFERENCES entities(id)
);

CREATE TABLE wisps (
    id INTEGER PRIMARY KEY,
    wisp_type TEXT NOT NULL,
    FOREIGN KEY(id) REFERENCES entities(id)
);

CREATE TABLE objectives (
    id INTEGER PRIMARY KEY,
    id_name TEXT NOT NULL,
    objective_type TEXT NOT NULL,
    activation_event TEXT NOT NULL,
    state TEXT NOT NULL,
    FOREIGN KEY(id) REFERENCES entities(id)
);

CREATE TABLE objective_kill_wisps (
    id INTEGER PRIMARY KEY,
    target_amount INTEGER NOT NULL,
    started_amount INTEGER NOT NULL,
    FOREIGN KEY(id) REFERENCES objectives(id)
);

CREATE TABLE stats (
    stat_name TEXT PRIMARY KEY,
    stat_value REAL NOT NULL
);

CREATE TABLE stock (
    resource_name TEXT PRIMARY KEY,
    amount INTEGER NOT NULL
);
