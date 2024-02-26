CREATE TABLE IF NOT EXISTS transformations (
  id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  from_piece BIGINT NOT NULL,
  to_piece BIGINT NOT NULL,
  quantity INT NOT NULL,
  cost MONEY NOT NULL,

  FOREIGN KEY(from_piece) REFERENCES pieces(id)
  ON DELETE CASCADE,

  FOREIGN KEY(to_piece) REFERENCES pieces(id)
  ON DELETE CASCADE,

  UNIQUE(from_piece, to_piece)
);

INSERT INTO transformations (from_piece, to_piece, quantity, cost)
SELECT p1.id AS from_piece, p2.id AS to_piece, 1, 45
FROM pieces as p1, pieces as p2
WHERE p1.name = 'P9' AND p2.name = 'P8';

INSERT INTO transformations (from_piece, to_piece, quantity, cost)
SELECT p1.id AS from_piece, p2.id AS to_piece, 1, 45
FROM pieces p1
JOIN pieces p2 On p1.id = p2.id
WHERE p1.name = 'P8' AND p2.name = 'P2';
