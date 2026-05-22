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

## Procedure

```text
1. Generate keypair (once, offline):

   ssh-keygen -t ed25519 -f catalog-signing -C 'leakferret catalog signing key'

   # Move private key to YubiKey / hardware HSM / air-gapped disk.
   # NEVER commit catalog-signing or catalog-signing.pub to git.

2. Embed public key in leakferret-core:

   # Read the public-key bytes
   ssh-keygen -e -m PKCS8 -f catalog-signing.pub  # for inspection
   # Convert to raw 32 bytes + base64-encode
   # Paste into crates/leakferret-core/src/catalog/signature.rs as
   # EMBEDDED_PUBLIC_KEY = Some("...")

3. Sign a new catalog version:

   cargo run --bin sign -- \
     --input  ../../catalog/2026.05.27.json \
     --key    /Volumes/secure/catalog-signing \
     --output ../../catalog/2026.05.27.json

4. Publish:

   # Upload signed JSON to https://catalog.<domain>/<version>.json
   # Update latest.json pointer
```

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
