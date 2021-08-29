use std::sync::Arc;
use std::time::SystemTime;

use rustls::{Certificate, ClientCertVerified, ClientCertVerifier, DistinguishedNames, DnsName, Error, HandshakeSignatureValid, RootCertStore, SignatureScheme};
use rustls::internal::msgs::handshake::{DigitallySignedStruct, DistinguishedName};

/// Turns off client authentication.
pub struct CustomClientAuth {}

impl CustomClientAuth {
    /// Constructs a `CustomClientAuth` and wraps it in an `Arc`.
    pub fn new() -> Arc<dyn ClientCertVerifier> {
        Arc::new(CustomClientAuth {})
    }
}

impl ClientCertVerifier for CustomClientAuth {
    fn offer_client_auth(&self) -> bool {
        true
    }

    fn client_auth_mandatory(&self, _sni: Option<&DnsName>) -> Option<bool> {
        Some(false)
    }

    fn client_auth_root_subjects(&self, sni: Option<&DnsName>) -> Option<DistinguishedNames> {
        Some(vec![])
    }

    fn verify_client_cert(
        &self,
        end_entity: &Certificate,
        intermediates: &[Certificate],
        sni: Option<&DnsName>,
        now: SystemTime,
    ) -> Result<ClientCertVerified, Error> {
        // TODO:
        Ok(ClientCertVerified::assertion())
    }
}