//! Release-only signature writer, wire-compatible with existing installations.
use base64::Engine;
fn sign() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).ok_or("Expected archive path")?;
    let encoded = std::env::var("TAURI_SIGNING_PRIVATE_KEY")?;
    let password = std::env::var("TAURI_SIGNING_PRIVATE_KEY_PASSWORD")?;
    let decoded = base64::engine::general_purpose::STANDARD.decode(encoded.trim())?;
    let secret = minisign::SecretKeyBox::from_string(std::str::from_utf8(&decoded)?)?
        .into_secret_key(Some(password))?;
    let signature = minisign::sign(None, &secret, std::fs::File::open(&path)?, None, None)?;
    let encoded = base64::engine::general_purpose::STANDARD.encode(signature.to_string());
    std::fs::write(format!("{path}.sig"), encoded)?;
    Ok(())
}
fn verify() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let archive = std::fs::read(args.get(2).ok_or("Expected archive")?)?;
    let signature = std::fs::read_to_string(args.get(3).ok_or("Expected signature")?)?;
    let public = std::fs::read_to_string(args.get(4).ok_or("Expected public key")?)?;
    let decoded = base64::engine::general_purpose::STANDARD.decode(signature.trim())?;
    let signature = minisign_verify::Signature::decode(std::str::from_utf8(&decoded)?)?;
    minisign_verify::PublicKey::decode(&public)?.verify(&archive, &signature, false)?;
    println!("Legacy 0.1.24 signature verified");
    Ok(())
}
fn main() {
    let result = if std::env::args().nth(1).as_deref() == Some("--verify") {
        verify()
    } else {
        sign()
    };
    if result.is_err() {
        eprintln!("Unable to sign or verify the update archive; verify release inputs.");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn signatures_are_accepted_by_the_legacy_verifier() {
        let keypair =
            minisign::KeyPair::generate_encrypted_keypair(Some("fixture-password".into())).unwrap();
        let encoded_secret = keypair.sk.to_box(None).unwrap().to_string();
        let secret = minisign::SecretKeyBox::from_string(&encoded_secret)
            .unwrap()
            .into_secret_key(Some("fixture-password".into()))
            .unwrap();
        let data = b"isolated update archive fixture";
        let signature = minisign::sign(None, &secret, std::io::Cursor::new(data), None, None)
            .unwrap()
            .to_string();
        let legacy_signature = minisign_verify::Signature::decode(&signature).unwrap();
        let public =
            minisign_verify::PublicKey::decode(&keypair.pk.to_box().unwrap().to_string()).unwrap();
        public.verify(data, &legacy_signature, false).unwrap();
        assert!(
            public
                .verify(b"modified", &legacy_signature, false)
                .is_err()
        );
    }
}
