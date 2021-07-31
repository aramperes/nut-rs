use crate::Config;

/// The certificate validation mechanism for NUT.
pub struct NutCertificateValidator {
    debug: bool,
}

impl NutCertificateValidator {
    /// Initialize a new instance.
    pub fn new(config: &Config) -> Self {
        NutCertificateValidator {
            debug: config.debug,
        }
    }
}

impl rustls::ServerCertVerifier for NutCertificateValidator {
    fn verify_server_cert(
        &self,
        _roots: &rustls::RootCertStore,
        presented_certs: &[rustls::Certificate],
        _dns_name: webpki::DNSNameRef<'_>,
        _ocsp: &[u8],
    ) -> Result<rustls::ServerCertVerified, rustls::TLSError> {
        // todo: verify certificates, but not hostnames

        if self.debug {
            let parsed = webpki::EndEntityCert::from(presented_certs[0].0.as_slice()).ok();
            if let Some(_parsed) = parsed {
                eprintln!("DEBUG <- Certificate received and parsed");
                // todo: reading values here... https://github.com/briansmith/webpki/pull/103
            } else {
                eprintln!("DEBUG <- Certificate not-parseable");
            }
        }

        // trust everything for now
        Ok(rustls::ServerCertVerified::assertion())
    }
}
