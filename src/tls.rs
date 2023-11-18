#[cfg(target_family = "windows")]
pub mod rustls {
    use s2n_quic::provider::tls::default::rustls::client::{
        ServerCertVerified, ServerCertVerifier,
    };
    use s2n_quic::provider::tls::default::rustls::ClientConfig;
    use s2n_quic::provider::tls::default::rustls::{Certificate, ServerName};
    use s2n_quic::provider::tls::default::Client;
    use std::error::Error;
    use std::sync::Arc;
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
        ) -> Result<ServerCertVerified, rustls::Error> {
            Ok(ServerCertVerified::assertion())
        }
    }
    pub fn insecure_client_tls(_: &str) -> Result<Client, Box<dyn Error>> {
        let verifier = Arc::new(NoCertVerifier {});
        let mut cb = ClientConfig::builder()
            .with_safe_default_cipher_suites()
            .with_safe_default_kx_groups()
            .with_safe_default_protocol_versions()?
            .with_custom_certificate_verifier(verifier.clone())
            .with_no_client_auth();

        cb.dangerous().set_certificate_verifier(verifier);

        let tls = Client::new(cb);
        return Ok(tls);
    }
}

#[cfg(target_family = "unix")]
pub mod s2ntls {
    use s2n_quic::provider::tls::default::callbacks::VerifyHostNameCallback;
    use s2n_quic::provider::tls::default::Client;
    use std::error::Error;
    use std::path::Path;
    pub struct InsecureTls {}
    impl VerifyHostNameCallback for InsecureTls {
        fn verify_host_name(&self, _host_name: &str) -> bool {
            return true;
        }
    }

    pub fn insecure_client_tls(cert: &str) -> Result<Client, Box<dyn Error>> {
        let tls = s2n_quic::provider::tls::default::Client::builder()
            .with_certificate(Path::new(cert))?
            .with_verify_host_name_callback(InsecureTls {})?
            .build()?;

        return Ok(tls);
    }
}
