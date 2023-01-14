use aerospike_tls::TlsConnector;

#[derive(Debug, Clone)]
pub struct TlsPolicy {
    pub tls_connector: TlsConnector,
}

impl TlsPolicy {
    pub fn new(tls_connector: TlsConnector) -> Self {
        Self { tls_connector, }
    }
}