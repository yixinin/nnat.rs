use s2n_tls::callbacks::VerifyHostNameCallback;

pub struct InsecureSkipVerify {}

impl VerifyHostNameCallback for InsecureSkipVerify {
    fn verify_host_name(&self, _: &str) -> bool {
        return true;
    }
}
