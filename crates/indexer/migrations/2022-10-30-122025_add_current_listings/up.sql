-- Your SQL goes here
-- table containing current marketplace listing information
CREATE TABLE current_marketplace_listings (
  -- sha256 of creator + collection_name + name
  token_data_id_hash VARCHAR(64) UNIQUE PRIMARY KEY NOT NULL,
  collection_data_id_hash VARCHAR(64) NOT NULL,
  market_address VARCHAR(66) NOT NULL,
  property_version NUMERIC NOT NULL,
  creator_address VARCHAR(66) NOT NULL,
  collection_name VARCHAR(128) NOT NULL,
  name VARCHAR(128) NOT NULL,
  seller VARCHAR(66) NOT NULL,
  amount NUMERIC NOT NULL,
  price NUMERIC NOT NULL,
  event_type VARCHAR(150) NOT NULL,
  inserted_at TIMESTAMP NOT NULL DEFAULT NOW(),
  last_transaction_version BIGINT NOT NULL
);
CREATE INDEX cml_tdih_pv_index ON current_marketplace_listings (token_data_id_hash, property_version);
CREATE INDEX cml_cdih_index ON current_marketplace_listings (collection_data_id_hash);
CREATE INDEX cml_insat_index ON current_marketplace_listings (inserted_at);
CREATE INDEX cml_tv_index ON current_marketplace_listings (last_transaction_version);
CREATE INDEX cml_seller_index ON current_marketplace_listings (seller);