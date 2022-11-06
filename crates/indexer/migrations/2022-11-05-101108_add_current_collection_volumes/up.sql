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
-- All collection volumes, at every buy/sell event (for price history)
CREATE TABLE collection_volumes (
  collection_data_id_hash VARCHAR(64) NOT NULL,
  volume NUMERIC NOT NULL,
  inserted_at TIMESTAMP NOT NULL DEFAULT NOW(),
  -- Last transaction version of the data in this table.
  last_transaction_version BIGINT NOT NULL,
  -- Constraints
  PRIMARY KEY (collection_data_id_hash)
);
CREATE INDEX cv_index ON collection_volumes (last_transaction_version);
-- Current token volumes
CREATE TABLE current_token_volumes (
  token_data_id_hash VARCHAR(64) NOT NULL,
  volume NUMERIC NOT NULL,
  inserted_at TIMESTAMP NOT NULL DEFAULT NOW(),
  -- Last transaction version of the data in this table.
  last_transaction_version BIGINT NOT NULL,
  -- Constraints
  PRIMARY KEY (token_data_id_hash)
);
CREATE INDEX ctv_index ON current_token_volumes (last_transaction_version);
-- All token volumes, at every buy/sell event (for price history)
CREATE TABLE token_volumes (
  token_data_id_hash VARCHAR(64) NOT NULL,
  volume NUMERIC NOT NULL,
  inserted_at TIMESTAMP NOT NULL DEFAULT NOW(),
  -- Last transaction version of the data in this table.
  last_transaction_version BIGINT NOT NULL,
  -- Constraints
  PRIMARY KEY (token_data_id_hash)
);
CREATE INDEX tv_index ON token_volumes (last_transaction_version);
-- Current daily collection volumes
-- CREATE TABLE current_daily_collection_volumes (
--   collection_data_id_hash VARCHAR(64) NOT NULL,
--   volume NUMERIC NOT NULL,
--   inserted_at TIMESTAMP NOT NULL DEFAULT NOW(),
--   -- Last transaction version of the data in this table.
--   last_transaction_version BIGINT NOT NULL,
--   -- Constraints
--   PRIMARY KEY (collection_data_id_hash)
-- );
-- CREATE INDEX ccv_index ON current_daily_collection_volumes (last_transaction_version);
-- -- Current weekly collection volumes
-- CREATE TABLE current_weekly_collection_volumes (
--   collection_data_id_hash VARCHAR(64) NOT NULL,
--   volume NUMERIC NOT NULL,
--   inserted_at TIMESTAMP NOT NULL DEFAULT NOW(),
--   -- Last transaction version of the data in this table.
--   last_transaction_version BIGINT NOT NULL,
--   -- Constraints
--   PRIMARY KEY (collection_data_id_hash)
-- );
-- CREATE INDEX ccv_index ON current_weekly_collection_volumes (last_transaction_version);
-- -- Current monthly collection volumes
-- CREATE TABLE current_monthly_collection_volumes (
--   collection_data_id_hash VARCHAR(64) NOT NULL,
--   volume NUMERIC NOT NULL,
--   inserted_at TIMESTAMP NOT NULL DEFAULT NOW(),
--   -- Last transaction version of the data in this table.
--   last_transaction_version BIGINT NOT NULL,
--   -- Constraints
--   PRIMARY KEY (collection_data_id_hash)
-- );
-- CREATE INDEX ccv_index ON current_monthly_collection_volumes (last_transaction_version);