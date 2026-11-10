# Contract Migration Rehearsal Command (`#1503`)

Loads representative stored state keys across schema versions (`v0`, `v1`, `v2`), runs every supported forward migration step up to `LATEST_STORAGE_SCHEMA_VERSION`, and reports any unmigrated or unrecognized keys.
