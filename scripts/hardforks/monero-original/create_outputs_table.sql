CREATE TABLE xmo_outputs
(
  address       VARCHAR(64),
  id            SERIAL PRIMARY KEY,
  amount        BIGINT,
  index         INTEGER,
  UNIQUE(amount, index)
)
