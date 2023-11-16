use rustls::client::{ServerCertVerified, ServerCertVerifier};
use rustls::Error;
use rustls::{Certificate, ServerName};
use std::time::SystemTime;
pub struct NoCertVerifier {}

impl ServerCertVerifier for NoCertVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &Certificate,
        _intermediates: &[Certificate],
        _server_name: &ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: SystemTime,
    ) -> Result<ServerCertVerified, Error> {
        Ok(ServerCertVerified::assertion())
    }
}
