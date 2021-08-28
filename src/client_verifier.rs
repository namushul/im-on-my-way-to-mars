use std::sync::Arc;
use std::time::SystemTime;

use rustls::{Certificate, ClientCertVerified, ClientCertVerifier, DistinguishedNames, DnsName, Error, RootCertStore, HandshakeSignatureValid, SignatureScheme};
use rustls::internal::msgs::handshake::{DistinguishedName, DigitallySignedStruct};

/// Turns off client authentication.
pub struct CustomClientAuth {
    roots: RootCertStore,
}

impl CustomClientAuth {
    /// Constructs a `NoClientAuth` and wraps it in an `Arc`.
    pub fn new(roots: RootCertStore) -> Arc<dyn ClientCertVerifier> {
        Arc::new(CustomClientAuth { roots })
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