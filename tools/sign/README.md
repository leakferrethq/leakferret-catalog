# sign — offline Ed25519 signing helper for the fixture catalog

Tiny standalone Rust binary that signs a `catalog/<version>.json` with
an Ed25519 private key. It depends on `leakferret-core` via a path
reference so the canonical-payload format (what bytes the signature
covers) is defined in exactly one place: `leakferret_core::catalog::
sign_catalog`. If that function changes upstream, rebuild this tool
and re-sign.

## Trust model

The private key MUST never live on a server, a CI runner, or any
machine that talks to the catalog CDN. Sign offline; publish online.
The matching public key gets embedded into `leakferret-core` and ships
with every binary release.

## Status

The signing keypair has been generated and the public key is embedded in
`leakferret-core`:

```
EMBEDDED_PUBLIC_KEY = Some("VxGTRy8eoWkb6k9s7noAbtSybHve4mGYymhV7y70cRI=")
```

So steps 1 and 2 below are already done. What remains for each catalog release
is signing (step 3) and publishing to the CDN (step 4), both done offline with
the private key — never on a CI runner.

## Procedure

```text
1. Generate keypair (DONE — once, offline):

   openssl genpkey -algorithm ed25519 -out catalog-ed25519.pem
   # Keep this PKCS#8 PEM private key on a hardware token / air-gapped disk.
   # NEVER commit it.

2. Embed public key in leakferret-core (DONE):

   openssl pkey -in catalog-ed25519.pem -pubout -outform DER | tail -c 32 | base64
   # -> pasted into crates/leakferret-core/src/catalog/signature.rs as
   #    EMBEDDED_PUBLIC_KEY, with the matching value asserted in its test.

3. Sign each dated catalog version (offline, on a machine with a Rust build):

   cargo run --bin sign -- \
     --input  ../../catalog/2026.05.27.json \
     --key    /path/to/catalog-ed25519.pem \
     --output ../../catalog/2026.05.27.json

   # Then update the latest.json pointer's checksum:
   sum=$(sha256sum ../../catalog/2026.05.27.json | awk '{print $1}')
   #   set "sha256" in ../../catalog/latest.json to $sum

4. Publish:

   # Upload the signed catalog/<version>.json and catalog/latest.json to
   # https://catalog.leakferret.com/ so `leakferret catalog refresh` can
   # fetch and verify them against the embedded public key.
```

## Rotating the key

Generate a fresh keypair, replace `EMBEDDED_PUBLIC_KEY` (and the expected value
in its test) in `leakferret-core`, re-sign every published catalog file with the
new private key, and cut a new engine release. Old binaries keep trusting the
old key, so keep the previous catalog signed with the old key until those
binaries age out.

## Key format

This binary expects a **PKCS#8 PEM-encoded** Ed25519 private key, i.e. a
file that starts with `-----BEGIN PRIVATE KEY-----`. Two ways to get
one:

* `openssl genpkey -algorithm ed25519 -out catalog-signing.pem` — writes
  PKCS#8 PEM directly. Recommended.
* `ssh-keygen -t ed25519 -f catalog-signing` then
  `ssh-keygen -p -N "" -m pkcs8 -f catalog-signing` to convert the
  OpenSSH key in place to PKCS#8 PEM.

The matching public key embedded in `leakferret-core` is **raw 32
bytes, base64-encoded** — not PEM. Use `openssl pkey -in
catalog-signing.pem -pubout -outform DER | tail -c 32 | base64` (or the
equivalent) to extract it.

## Why a separate tool

The sign function is tiny — a dozen lines wrapping `ed25519-dalek` plus
JSON IO. Reasons it gets its own binary:

* The signing path uses a private key. The verification path uses a
  public key. Keeping signing physically separate from the engine
  binary makes it harder to accidentally ship something that links the
  private key into the same process as a network client.
* The catalog repo can run `cargo run --bin sign` without depending on
  the full `leakferret` workspace (only the one path-dep crate).
* If we ever swap signature schemes (e.g. cosign, sigstore), only this
  one binary changes — the engine swap follows separately.
