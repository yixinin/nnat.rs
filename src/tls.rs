use rustls::client::{ServerCertVerified, ServerCertVerifier};
use rustls::Error;
use rustls::{Certificate, ServerName};
use std::time::SystemTime;
pub struct NoCertVerifier {}

impl ServerCertVerifier for NoCertVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &Certificate,
        intermediates: &[Certificate],
        server_name: &ServerName,
        scts: &mut dyn Iterator<Item = &[u8]>,
        ocsp_response: &[u8],
        now: SystemTime,
    ) -> Result<ServerCertVerified, Error> {
        Ok(ServerCertVerified::assertion())
    }
}
