CREATE VIEW order_details AS
SELECT
  c.name AS client_name,
  o.number,
  p.name AS piece_name,
  o.quantity,
  o.due_date,
  o.early_pen,
  o.late_pen
FROM orders o
INNER JOIN clients c ON c.id = o.client_id
INNER JOIN pieces p ON p.id = o.piece_id;
