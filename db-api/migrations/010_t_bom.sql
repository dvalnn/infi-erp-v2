CREATE TABLE IF NOT EXISTS bom (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,

    order_id BIGINT NOT NULL,
    transformation_id BIGINT NOT NULL,
    piece_number INT NOT NULL,
    pieces_total INT NOT NULL,
    step_number INT NOT NULL,
    steps_total INT NOT NULL,

    FOREIGN KEY (order_id) REFERENCES orders (id)
    ON DELETE CASCADE,

    FOREIGN KEY (transformation_id) REFERENCES transformations (id)
    On DELETE CASCADE,

    CHECK (piece_number > 0 AND piece_number <= pieces_total)
    CHECK (pieces_total > 0),
    CHECK (step_number > 0 AND step_number <= steps_total)
    CHECK (steps_total > 0)

    UNIQUE (order_id, transformation_id, piece_number, step_number)
);
