use s2n_tls::callbacks::VerifyHostNameCallback;

pub struct IgnoreTls {}

impl VerifyHostNameCallback for IgnoreTls {
    fn verify_host_name(&self, host_name: &str) -> bool {
        return true;
    }
}
