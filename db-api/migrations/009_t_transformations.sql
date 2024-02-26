CREATE DOMAIN tools AS VARCHAR(2) CHECK (value IN(
  'T1',
  'T2',
  'T3',
  'T4',
  'T5',
  'T6'
));

CREATE TABLE IF NOT EXISTS transformations (
  id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  from_piece BIGINT NOT NULL,
  to_piece BIGINT NOT NULL,
  tool tools NOT NULL,
  quantity INT NOT NULL,
  cost MONEY NOT NULL,

  FOREIGN KEY(from_piece) REFERENCES pieces(id)
  ON DELETE CASCADE,

  FOREIGN KEY(to_piece) REFERENCES pieces(id)
  ON DELETE CASCADE,

  UNIQUE(from_piece, to_piece, tool)
);

-- P9 from P8
INSERT INTO transformations (from_piece, to_piece, tool, quantity, cost)
SELECT f.id AS from_piece, t.id AS to_piece, 'T5', 1, 45
FROM pieces as f, pieces as t
WHERE f.name = 'P8' AND t.name = 'P9';

-- P7 from P8
INSERT INTO transformations (from_piece, to_piece, tool, quantity, cost)
SELECT f.id AS from_piece, t.id AS to_piece, 'T6', 1, 15
FROM pieces as f, pieces as t
WHERE f.name = 'P8' AND t.name = 'P7';

-- P8 from P2
INSERT INTO transformations (from_piece, to_piece, tool, quantity, cost)
SELECT f.id AS from_piece, t.id AS to_piece, 'T1', 1, 45
FROM pieces as f, pieces as t
WHERE f.name = 'P2' AND t.name = 'P8';

-- P7 from P4
INSERT INTO transformations (from_piece, to_piece, tool, quantity, cost) 
SELECT f.id AS from_piece, t.id AS to_piece, 'T3', 1, 15
FROM pieces as f, pieces as t
WHERE f.name = 'P4' AND t.name = 'P7';

-- P6 from P4
INSERT INTO transformations (from_piece, to_piece, tool, quantity, cost)
SELECT f.id AS from_piece, t.id AS to_piece, 'T2', 1, 25
FROM pieces as f, pieces as t
WHERE f.name = 'P4' AND t.name = 'P6';

-- P5 from P4
INSERT INTO transformations (from_piece, to_piece, tool, quantity, cost)
SELECT f.id AS from_piece, t.id AS to_piece, 'T4', 1, 25
FROM pieces as f, pieces as t
WHERE f.name = 'P4' AND t.name = 'P5';

-- P4 from P3 (T3)
INSERT INTO transformations (from_piece, to_piece, tool, quantity, cost)
SELECT f.id AS from_piece, t.id AS to_piece, 'T3', 1, 25
FROM pieces as f, pieces as t
WHERE f.name = 'P3' AND t.name = 'P4';

-- P4 from P3 (T2)
INSERT INTO transformations (from_piece, to_piece, tool, quantity, cost)
SELECT f.id AS from_piece, t.id AS to_piece, 'T2', 1, 15
FROM pieces as f, pieces as t
WHERE f.name = 'P3' AND t.name = 'P4';

-- P3 from P1
INSERT INTO transformations (from_piece, to_piece, tool, quantity, cost)
SELECT f.id AS from_piece, t.id AS to_piece, 'T1', 1, 45
FROM pieces as f, pieces as t
WHERE f.name = 'P1' AND t.name = 'P3';

