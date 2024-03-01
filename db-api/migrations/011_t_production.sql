CREATE IF NOT EXISTS TABLE production {

  id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  order_id BIGINT NOT NULL,
  bom_id BIGINT NOT NULL,
  day INT NOT NULL,
  timeslot INT NOT NULL,
  prod_line INT NOT NULL,

  FOREIGN KEY (order_id) REFERENCES orders (id)
  ON DELETE CASCADE,
  FOREIGN KEY (bom_id) REFERENCES bom (id)
  ON DELETE CASCADE,

  CHECK (day > 0),
  CHECK (timeslot > 0 AND timeslot <= 12),
  CHECK (prod_line > 0 AND prod_line <= 6),
}
