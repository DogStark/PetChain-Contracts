# Claim-Document Status, Versions and Revocation (Issue #1341)

A claim document is a SHA-256 digest registered against a `claim_id`. Every
registered digest gets an immutable-identity record
(`ClaimDocumentRecord`) with a status and a version.

## Statuses

| Status       | Meaning                                    | Can back a new approval? |
|--------------|--------------------------------------------|--------------------------|
| `Active`     | Current version of the document            | Yes                      |
| `Superseded` | Replaced by `superseded_by`                | No                       |
| `Revoked`    | Invalidated; terminal                      | No                       |

Transitions:

```text
submit_claim_document ──► Active ──supersede_claim_document──► Superseded
                            │                                      │
                            └────revoke_claim_document─────► Revoked ◄┘
```

Any transition out of `Revoked` fails with `ClaimDocumentRevoked` (174).
Superseding a non-`Active` document fails with `ClaimDocumentSuperseded`
(175) or `ClaimDocumentRevoked` (174).

## Versions

An original submission is version 1. `supersede_claim_document` appends a
new record with `version = old.version + 1` and links both ways
(`supersedes` / `superseded_by`). Old versions are never deleted.

## Authorization

| Operation                  | Who                                   |
|----------------------------|---------------------------------------|
| `submit_claim_document`    | Owner of the pet the claim is bound to |
| `supersede_claim_document` | Owner of the bound pet                |
| `revoke_claim_document`    | Owner of the bound pet, or an admin   |
| `approve_claim`            | Admin                                 |

The first submission binds `claim_id` to a pet; later submissions for another
pet fail with `ClaimPetMismatch` (177). All operations require the caller's
signature.

## Revoked documents: auditable but never newly accepted

- The revoked record keeps its digest, version, submitter, revoker
  (`status_changed_by`), time and `revocation_reason`.
- `verify_claim_document` is a pure integrity check and still returns `true`
  for a matching digest regardless of status. Use
  `is_claim_document_acceptable` to find out whether the document could back
  an approval.
- Revocation **burns the digest contract-wide**: it can never be submitted
  again, on the same claim or on any other, and never used as a
  supersession target. This stops a revoked document from being
  re-registered to satisfy an approval.

## Approval and historical settlement behavior

`approve_claim(approver, claim_id, doc_indices)` requires every listed index
(non-empty, strictly ascending) to be `Active` and not burned. It writes an
immutable `ClaimSettlement` with the digests and versions that were accepted.

Once settled:

- A second approval, new submissions and supersessions fail with
  `ClaimAlreadySettled` (176).
- Revocation remains possible (for example, fraud discovered later). It does
  **not** unwind or modify the settlement: `get_claim_settlement` keeps
  returning exactly what the approver accepted.
- `settlement_has_revoked_documents(claim_id)` returns `true` when any
  document captured in the settlement has since been revoked, so auditors can
  flag the claim for off-chain review.
