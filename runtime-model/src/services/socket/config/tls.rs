use std::{
	io::Cursor,
	sync::Arc,
};

use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use serde::{Deserialize, Deserializer, de};
use rustls::{ServerConfig};
use tokio_rustls::TlsAcceptor;

use config_crap::{WithEnv};

#[derive(PartialEq,Debug)]
pub struct PemPrivateKey(pub PrivateKeyDer<'static>);
impl Clone for PemPrivateKey {
	fn clone(&self) -> Self {
		Self(self.0.clone_key())
	}
}
impl<'de> Deserialize<'de> for PemPrivateKey {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let pem = String::deserialize(d)?;
        let key = rustls_pemfile::private_key(&mut Cursor::new(pem.as_bytes()))
            .map_err(de::Error::custom)?
            .ok_or_else(|| de::Error::custom("no private key found in PEM data"))?;
        Ok(Self(key))
    }
}

#[derive(PartialEq,Debug,Clone)]
pub struct PemCertChain(pub Vec<CertificateDer<'static>>);
impl<'de> Deserialize<'de> for PemCertChain {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let pem = String::deserialize(d)?;
        let certs: Result<Vec<_>, _> =
            rustls_pemfile::certs(&mut Cursor::new(pem.as_bytes())).collect();
        let certs = certs.map_err(de::Error::custom)?;
        if certs.is_empty() {
            return Err(de::Error::custom("certificate chain is empty"));
        }
        Ok(Self(certs))
    }
}

#[derive(Deserialize,PartialEq,Debug,Clone)]
pub enum OnlyMode {
    #[serde(rename = "tlsv1.3")]
    V1_3,
    #[serde(rename = "tlsv1.2")]
    V1_2,
}

#[derive(Deserialize,PartialEq,Debug,Clone)]
pub struct RusTLSServerConfig {
    pub only: Option<OnlyMode>,
    pub key: WithEnv<PemPrivateKey>,
    pub chain: WithEnv<PemCertChain>,
}
impl RusTLSServerConfig {
    pub fn build(&self, alpn: Vec<Vec<u8>>) -> Result<TlsAcceptor, anyhow::Error> {
        let versions: &[&'static rustls::SupportedProtocolVersion] = match self.only {
            None => &[&rustls::version::TLS13, &rustls::version::TLS12],
            Some(OnlyMode::V1_3) => &[&rustls::version::TLS13],
            Some(OnlyMode::V1_2) => &[&rustls::version::TLS12],
        };
        let mut config = ServerConfig::builder_with_protocol_versions(versions)
            .with_no_client_auth()
            .with_single_cert(self.chain.clone().into_inner().0, self.key.clone().into_inner().0)?;
        config.alpn_protocols = alpn;
        Ok(TlsAcceptor::from(Arc::new(config)))
    }
}
