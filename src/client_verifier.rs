use std::sync::Arc;
use std::time::SystemTime;

use rustls::{Certificate, ClientCertVerified, ClientCertVerifier, DistinguishedNames, DnsName, Error};

/// Enables client authentication without verifying client certificates.
pub struct AcceptAnyClientCert {}

impl AcceptAnyClientCert {
    pub fn new() -> Arc<dyn ClientCertVerifier> {
        Arc::new(AcceptAnyClientCert {})
    }
}

impl ClientCertVerifier for AcceptAnyClientCert {
    fn offer_client_auth(&self) -> bool {
        true
    }

    fn client_auth_mandatory(&self, _sni: Option<&DnsName>) -> Option<bool> {
        Some(false)
    }

    fn client_auth_root_subjects(&self, _sni: Option<&DnsName>) -> Option<DistinguishedNames> {
        Some(vec![])
    }

    fn verify_client_cert(
        &self,
        _end_entity: &Certificate,
        _intermediates: &[Certificate],
        _sni: Option<&DnsName>,
        _now: SystemTime,
    ) -> Result<ClientCertVerified, Error> {
        // TODO: Replace this tls with openssl tls
        Ok(ClientCertVerified::assertion())
    }
}