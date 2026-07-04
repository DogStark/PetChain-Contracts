-- Migration 005 down: no-op.
-- Encryption of existing secrets is a manual operational step (see up script).
-- Rolling back this migration does not decrypt existing rows; that must be
-- done manually using the reverse of the encrypt_existing_secrets.py script.
SELECT 1;
