-- Add encrypted credentials columns to hosts table

ALTER TABLE hosts ADD COLUMN credentials_encrypted BLOB;
ALTER TABLE hosts ADD COLUMN credentials_iv BLOB;
