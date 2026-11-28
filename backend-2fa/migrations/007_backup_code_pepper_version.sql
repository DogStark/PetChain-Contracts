-- Existing Argon2id digests are version 0 until a new code set is issued.
-- Derive the version on every write, including atomic recovery-code removal.
ALTER TABLE user_two_factor ADD COLUMN backup_code_pepper_version BIGINT
    GENERATED ALWAYS AS (
        CASE WHEN (backup_codes::jsonb ->> 0) ~ '^hmac-sha256:[0-9]{1,10}:'
            THEN split_part(backup_codes::jsonb ->> 0, ':', 2)::bigint
            ELSE 0
        END
    ) STORED;
