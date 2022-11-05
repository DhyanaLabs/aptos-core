-- Your SQL goes here
-- Current collection volumes
CREATE TABLE current_collection_volumes (
  collection_data_id_hash VARCHAR(64) NOT NULL,
  volume NUMERIC NOT NULL,
  inserted_at TIMESTAMP NOT NULL DEFAULT NOW(),
  -- Last transaction version of the data in this table.
  last_transaction_version BIGINT NOT NULL,
  -- Constraints
  PRIMARY KEY (collection_data_id_hash)
);
CREATE INDEX ccv_index ON current_collection_volumes (last_transaction_version);