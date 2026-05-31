//! Self-signed certificate generation.
//!
//! Replaces the JKS keystore approach from the Java version — here we
//! generate certs at runtime via `rcgen`.

use crate::error::{NetworkError, Result};
use rcgen::{CertificateParams, DistinguishedName, DnType, KeyPair};

/// Generate a self-signed certificate and private key for a node.
///
/// `subject_name` example: `"commander-aylin"` — appears as a SAN entry
/// during the TLS handshake.
///
/// Returns `(PEM-encoded cert, PEM-encoded key)`.
pub fn generate_self_signed(subject_name: &str) -> Result<(String, String)> {
    let mut params = CertificateParams::new(vec![subject_name.to_string()])
        .map_err(|e| NetworkError::Certificate(e.to_string()))?;

    let mut dn = DistinguishedName::new();
    dn.push(DnType::CommonName, subject_name);
    dn.push(DnType::OrganizationName, "Military P2P");
    params.distinguished_name = dn;

    let key_pair = KeyPair::generate().map_err(|e| NetworkError::Certificate(e.to_string()))?;

    let cert = params
        .self_signed(&key_pair)
        .map_err(|e| NetworkError::Certificate(e.to_string()))?;

    Ok((cert.pem(), key_pair.serialize_pem()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_valid_pem() {
        let (cert_pem, key_pem) = generate_self_signed("test-node").unwrap();
        assert!(cert_pem.contains("BEGIN CERTIFICATE"));
        assert!(cert_pem.contains("END CERTIFICATE"));
        assert!(key_pem.contains("BEGIN PRIVATE KEY"));
        assert!(key_pem.contains("END PRIVATE KEY"));
    }

    #[test]
    fn each_call_produces_different_keys() {
        let (_, k1) = generate_self_signed("node-a").unwrap();
        let (_, k2) = generate_self_signed("node-b").unwrap();
        assert_ne!(k1, k2);
    }
}
