# leakferret-catalog

Versioned, **signed** catalog of known-public credentials that look
real but aren't secrets — Stripe published test keys, AWS canary
patterns (`AKIAIOSFODNN7EXAMPLE`), RFC example keys, `jwt.io` sample
tokens, and similar documented placeholders.

[`leakferret`](https://github.com/leakferrethq/leakferret) loads this
catalog and matches every candidate against it *before* any classifier
or verifier runs. A hit produces a deterministic `FIXTURE` verdict in
microseconds — no model call, no network request.

- Project: <https://leakferret.com>
- Engine + CLI + MCP server: <https://github.com/leakferrethq/leakferret>
- Code license: MIT · **Data license: CC-BY-SA-4.0**

## What the fixture catalog is

A scanner that flags secrets is only trustworthy if it does *not* flag
the famous fake ones. trufflehog will happily verify Stripe's published
test key as a live key — because it *is* a live test key. It reports it
as a finding, the developer chases it, discovers it's a documented
placeholder, and loses trust in the tool.

This catalog is the surgical "no, that's a fixture" layer. It is a list
of credential values (and credential *shapes*) that are public by
design, each with a citation to where the vendor or an RFC published
it. When `leakferret` sees one, it marks the finding `FIXTURE` and moves
on instead of paging a human.

A bundled snapshot ships inside every `leakferret` release, so the tool
is useful with zero network access. CDN updates are picked up by
`leakferret catalog refresh` without a tool upgrade.

## Anatomy of an entry

Each entry is a JSON object in `catalog/<YYYY.MM.DD>.json`:

```json
{
  "id": "aws.iam.docs.AKIAIOSFODNN7EXAMPLE",
  "kind": "aws_access_key",
  "match": { "strategy": "exact", "value": "AKIAIOSFODNN7EXAMPLE" },
  "source": "https://docs.aws.amazon.com/IAM/latest/UserGuide/id_credentials_access-keys.html",
  "source_checked_at": "2026-04-01",
  "rationale": "AWS docs canary access key ID used in every IAM example.",
  "trust": "vendor_published",
  "verdict": "FIXTURE"
}
```

### Trust levels

| level | meaning |
|---|---|
| `vendor_published` | Published in the vendor's own docs (highest trust) |
| `rfc_published` | Published in an IETF / W3C document |
| `community_verified` | Community PR + two reviewer sign-offs |
| `community_unverified` | Community PR, no sign-off (off by default) |

### Match strategies

* `exact` — cleartext exact match. Used for credentials that are public
  by design (Stripe test keys, AWS canary).
* `exact_hash` — SHA-256 of the credential. Used when shipping the
  cleartext would be undesirable (large keys, OpenSSH test keys embedded
  in source trees, internal honeytokens).
* `regex` — pattern match. Used for placeholder *shapes*
  (`AIza…PLACEHOLDER…`, `sk-xxxxx…`).

### Honeytokens

Entries with `verdict: "HONEYTOKEN"` invert the logic: a match *raises*
alert severity instead of lowering it. A honeytoken hit means someone is
reading source they shouldn't.

## Contribution rule: a source URL is required

Catalog entries must be reproducible by an outside reviewer. **Every
`vendor_published` or `rfc_published` entry must cite a stable
`source` URL where the credential is publicly documented, plus the
`source_checked_at` date you verified it.** No URL, no merge.

This rule is the whole point of the catalog: we are not collecting
"strings that look fake," we are collecting credentials a third party
can confirm are intentionally public. If you can't link to where the
vendor or an RFC published it, it doesn't belong here.

To contribute:

1. Fork the repo.
2. Add an entry to the latest `catalog/<YYYY.MM.DD>.json`. Versions are
   append-only — don't edit older files.
3. Confirm the `source` URL resolves and the credential is published by
   the vendor or in an RFC. Set `source_checked_at` to today.
4. Write a `rationale` that states what the source is and why the
   credential is documented (test key, canary, example).
5. Use a `kind` that already exists in
   `leakferret-core/src/patterns/registry.rs`.
6. Open a PR. **Two reviewers (other than the author) must sign off**
   before merge.

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for the full checklist.

```bash
# Hash a value for an exact_hash entry
echo -n "your-value-here" | sha256sum

# Test your entry against the catalog locally
cargo run -p leakferret-cli -- catalog test "sk_test_4eC39HqLyjWDarjtT1zdp7dc" \
  --file catalog/2026.05.27.json
```

## Signing model (Ed25519)

Catalog files are signed with the project's **Ed25519** key so a
tampered or spoofed catalog can't downgrade a real finding to a fixture.

- **Signing is offline.** The private key never lives on a server, a CI
  runner, or any machine that talks to the catalog CDN. It stays on a
  hardware token / air-gapped disk. Sign offline; publish online.
- **Verification is built in.** The matching public key — raw 32 bytes,
  base64-encoded — is embedded in `leakferret-core` and ships with every
  binary release. The engine verifies the signature on load before
  trusting any entry.
- **One canonical payload.** The exact bytes a signature covers are
  defined once, in `leakferret_core::catalog::sign_catalog`. The signing
  helper in [`tools/sign/`](tools/sign/) depends on `leakferret-core` by
  path so the format can never drift between signer and verifier.

A new catalog version (`YYYY.MM.DD.json`) is cut per merge batch, signed,
then uploaded. See [`tools/sign/README.md`](tools/sign/README.md) for the
full key-generation and signing procedure.

```bash
cargo run --bin sign -- \
  --input  catalog/2026.05.27.json \
  --key    /path/to/offline/catalog-signing.pem \
  --output catalog/2026.05.27.json
```

## Distribution

Catalog files are served at
`https://catalog.leakferret.com/{version}.json`, with `latest.json`
pointing at the current version and its SHA-256. The bundled snapshot in
each `leakferret` release is the offline fallback; `leakferret catalog
refresh` pulls newer signed versions from the CDN without a tool upgrade.

```bash
leakferret catalog refresh
```

## License

The catalog **data** in this repository is licensed under
**CC-BY-SA-4.0** (Creative Commons Attribution-ShareAlike 4.0
International). You are free to share and adapt it, even commercially,
provided you give attribution and license derivatives under the same
terms. See [`LICENSE`](LICENSE).

The signing helper and other code in this repo are MIT-licensed,
matching the [`leakferret`](https://github.com/leakferrethq/leakferret)
engine.

Maintainer: Maria Khan &lt;missusk@protonmail.com&gt;
