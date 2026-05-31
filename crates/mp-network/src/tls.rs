//! rustls server/client configuration.
//!
//! Prepared for production use but not currently wired into [`Server::run`].

use crate::error::{NetworkError, Result};
use rustls::pki_types::{pem::PemObject, CertificateDer, PrivateKeyDer};
use rustls::ServerConfig;
use std::sync::Arc;

/// Build a rustls `ServerConfig` from PEM-encoded cert and key.
pub fn build_server_config(cert_pem: &str, key_pem: &str) -> Result<Arc<ServerConfig>> {
    let cert_chain: Vec<CertificateDer> = CertificateDer::pem_slice_iter(cert_pem.as_bytes())
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| NetworkError::Tls(format!("cert parse: {}", e)))?;

    if cert_chain.is_empty() {
        return Err(NetworkError::Tls("no certificates found in PEM".into()));
    }

    let key = PrivateKeyDer::from_pem_slice(key_pem.as_bytes())
        .map_err(|e| NetworkError::Tls(format!("key parse: {}", e)))?;

    let config = ServerConfig::builder()
        .with_no_client_auth() // mTLS to be added later
        .with_single_cert(cert_chain, key)
        .map_err(|e| NetworkError::Tls(format!("config build: {}", e)))?;

    Ok(Arc::new(config))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cert::generate_self_signed;

    #[test]
    fn builds_config_from_generated_cert() {
        let (cert, key) = generate_self_signed("test").unwrap();
        let config = build_server_config(&cert, &key).unwrap();
        // Builds without panic.
        assert!(Arc::strong_count(&config) >= 1);
    }
}
