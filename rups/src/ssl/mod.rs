use crate::Config;

/// The certificate validation mechanism that allows any certificate.
pub struct InsecureCertificateValidator {
    debug: bool,
}

impl InsecureCertificateValidator {
    /// Initialize a new instance.
    pub fn new(config: &Config) -> Self {
        InsecureCertificateValidator {
            debug: config.debug,
        }
    }
}

impl rustls::ServerCertVerifier for InsecureCertificateValidator {
    fn verify_server_cert(
        &self,
        _roots: &rustls::RootCertStore,
        _presented_certs: &[rustls::Certificate],
        _dns_name: webpki::DNSNameRef<'_>,
        _ocsp: &[u8],
    ) -> Result<rustls::ServerCertVerified, rustls::TLSError> {
        if self.debug {
            eprintln!("DEBUG <- (!) Certificate received, but not verified");
        }
        Ok(rustls::ServerCertVerified::assertion())
    }
}
