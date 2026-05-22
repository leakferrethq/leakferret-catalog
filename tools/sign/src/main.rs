//! `leakferret-catalog-sign` — sign a catalog JSON with an Ed25519
//! private key.
//!
//! Reuses [`leakferret_core::catalog::sign_catalog`] so that the
//! canonical-payload format stays in lockstep with the verifier in
//! `leakferret-core`. There is exactly one implementation of "what
//! bytes get signed", and it lives in the engine crate.
//!
//! The private key is expected to be a PKCS#8 PEM-encoded Ed25519
//! key. Keep it on hardware or an air-gapped disk — see
//! `tools/sign/README.md` for the full key-management procedure.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use ed25519_dalek::pkcs8::DecodePrivateKey;
use ed25519_dalek::SigningKey;

use leakferret_core::catalog::{sign_catalog, CatalogFile};

#[derive(Debug, Parser)]
#[command(
    name = "sign",
    version,
    about = "Sign a leakferret fixture catalog with an Ed25519 private key.",
    long_about = "Reads <input>, signs it with the PKCS#8 PEM key at <key>, \
                  writes the signed JSON to <output>. Use --output equal to \
                  --input to sign in place."
)]
struct Cli {
    /// Catalog JSON file to sign.
    #[arg(long)]
    input: PathBuf,

    /// PKCS#8 PEM-encoded Ed25519 private key. Keep on hardware / HSM
    /// / air-gapped disk. NEVER commit.
    #[arg(long)]
    key: PathBuf,

    /// Output path for the signed catalog. Same as --input to sign in
    /// place.
    #[arg(long)]
    output: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let raw = std::fs::read_to_string(&cli.input)
        .with_context(|| format!("read {}", cli.input.display()))?;
    let mut file: CatalogFile = serde_json::from_str(&raw)
        .with_context(|| format!("parse catalog from {}", cli.input.display()))?;

    // Always clear any pre-existing signature before signing so a
    // re-sign produces the same canonical payload as a first sign.
    file.signature = None;

    let pem = std::fs::read_to_string(&cli.key)
        .with_context(|| format!("read key {}", cli.key.display()))?;
    let signing_key = SigningKey::from_pkcs8_pem(&pem)
        .with_context(|| format!("decode PKCS#8 PEM from {}", cli.key.display()))?;

    let signature = sign_catalog(&file, &signing_key)
        .context("sign canonical catalog payload")?;
    file.signature = Some(signature);

    let signed = serde_json::to_string_pretty(&file)
        .context("serialise signed catalog")?;
    std::fs::write(&cli.output, format!("{signed}\n"))
        .with_context(|| format!("write {}", cli.output.display()))?;

    println!(
        "signed {} ({} entries) -> {}",
        cli.input.display(),
        file.entries.len(),
        cli.output.display()
    );
    Ok(())
}
