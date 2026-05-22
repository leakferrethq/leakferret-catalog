# Contributing a catalog entry

Catalog entries must be reproducible by an outside reviewer. Use this
checklist for every PR.

## Required for `vendor_published` and `rfc_published`

- [ ] The credential string is published in the vendor's docs or an
      RFC, with a stable URL.
- [ ] You include `source` (URL) and `source_checked_at` (today's date).
- [ ] The `rationale` says what the source is and why the credential
      is documented (test key, canary, example).
- [ ] The `kind` field uses one of the existing kinds in
      `leakferret-core/src/patterns/registry.rs`.

## Required for `community_verified`

- [ ] Two reviewers (other than the author) sign off.
- [ ] You explain in the PR description why this credential is safe
      to publish in cleartext (or why you chose `exact_hash`).

## Match strategy

- Use `exact` for credentials that are public by design.
- Use `regex` for *patterns* of placeholders.
- Use `exact_hash` when you want the catalog to recognise a value
  without shipping the cleartext (large keys, internal honeytokens).

## Honeytokens

Honeytokens use `verdict: "HONEYTOKEN"` instead of `"FIXTURE"`. They
*raise* alert severity rather than downgrade it — matches indicate
someone is reading source they shouldn't.

## Local verification

```bash
# Hash a value
echo -n "your-value-here" | sha256sum

# Test your entry
cargo run -p leakferret-cli -- catalog test "your-value-here" \
  --file catalog/2026.05.27.json
```

## Versioning

We cut a new catalog version (`YYYY.MM.DD.json`) per merge batch.
Don't edit older versions; entries are append-only across versions.
