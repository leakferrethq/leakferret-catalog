# leakferret-catalog

Versioned, signed catalog of **known-public** credentials that look
real but aren't secrets — Stripe published test keys, AWS canary
patterns, RFC example keys, JWT.io examples, etc.

`leakferret` loads this catalog and matches every candidate against
it *before* any classifier or verifier runs. A hit produces a
deterministic `FIXTURE` verdict in microseconds.

## Why this exists

Trufflehog will happily verify Stripe's published test key as a live
key (because it *is* a live test key). It reports it as a finding,
the developer chases it, finds it's a documented placeholder, and
loses trust in the scanner. This catalog is the surgical "no, that's
a fixture" layer.

## Trust levels

| level | meaning |
|---|---|
| `vendor_published` | Published in the vendor's own docs (highest trust) |
| `rfc_published` | Published in an IETF / W3C document |
| `community_verified` | Community PR + two reviewer sign-offs |
| `community_unverified` | Community PR, no sign-off (off by default) |

## Match strategies

* `exact` — cleartext exact match. Used for credentials that are
  public by design (Stripe test keys, AWS canary).
* `exact_hash` — SHA-256 of the credential. Used when shipping the
  cleartext would be undesirable (large keys, OpenSSH test keys
  embedded in source trees).
* `regex` — pattern match. Used for placeholder shapes
  (`AIza…PLACEHOLDER…`, `sk-xxxxx…`).

## Updating

```bash
# Local development:
cargo run -p leakferret-cli -- catalog test "sk_test_4eC39HqLyjWDarjtT1zdp7dc" --file catalog/2026.05.27.json

# Refresh from CDN:
leakferret catalog refresh
```

## Contributing an entry

1. Fork.
2. Add an entry to `catalog/<latest>.json`.
3. Verify the source URL works and the credential is **published**
   by the vendor or in an RFC.
4. Open a PR. Two reviewers must sign off before merge.

## License

CC-BY-SA-4.0 for the data. Reuse freely; derivatives must remain open.

## Distribution

Catalog files are served at `https://catalog.leakferret.dev/{version}.json`
(via Cloudflare Pages, signed with the project Ed25519 key — public
key shipped in `leakferret-core`).

A bundled snapshot ships with every `leakferret` release; CDN updates
are picked up by `leakferret catalog refresh` without a tool upgrade.
