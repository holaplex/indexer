//! AssetIdentifier utils - Parse and capture tx and cid
use base64::display::Base64Display;
use cid::Cid;
use url::Url;
/// An Arweave transaction ID
#[derive(Debug, Clone, Copy)]
pub struct ArTxid(pub [u8; 32]);

/// Struct to hold tx ids
#[derive(Debug, Clone, Copy)]
pub struct AssetIdentifier {
    /// ipfs cid
    pub ipfs: Option<Cid>,
    /// Arweave tx id
    pub arweave: Option<ArTxid>,
}

impl AssetIdentifier {
    fn visit_url(url: &Url, mut f: impl FnMut(&str)) {
        Some(url.scheme())
            .into_iter()
            .chain(url.domain().into_iter().flat_map(|s| s.split('.')))
            .chain(Some(url.username()))
            .chain(url.password())
            .chain(Some(url.path()))
            .chain(url.path_segments().into_iter().flatten())
            .chain(url.query())
            .chain(url.fragment().into_iter().flat_map(|s| s.split('/')))
            .for_each(&mut f);

        url.query_pairs().for_each(|(k, v)| {
            f(k.as_ref());
            f(v.as_ref());
        });
    }

    fn try_ipfs(s: &str) -> Option<Cid> {
        s.try_into().ok()
    }

    fn try_arweave(s: &str) -> Option<ArTxid> {
        [
            base64::URL_SAFE,
            base64::URL_SAFE_NO_PAD,
            base64::STANDARD,
            base64::STANDARD_NO_PAD,
        ]
        .into_iter()
        .find_map(|c| {
            base64::decode_config(s.as_bytes(), c)
                .ok()
                .and_then(|v| v.try_into().ok())
                .map(ArTxid)
        })
    }

    fn advance_heuristic<T>(state: &mut Result<Option<T>, ()>, value: T) {
        match state {
            Ok(None) => *state = Ok(Some(value)),
            Ok(Some(_)) => *state = Err(()),
            Err(()) => (),
        }
    }
    /// Extract cid from asset
    pub fn get_cid_with_svc(&self) -> String {
        if self.arweave.is_some() {
            format!(
                "arweave/{}",
                Base64Display::with_config(&self.arweave.unwrap().0, base64::URL_SAFE_NO_PAD)
                    .to_string()
            )
        } else {
            format!("ipfs/{}", self.ipfs.unwrap().to_string())
        }
    }
    /// parse cid from url
    pub fn new(url: &Url) -> Self {
        let mut ipfs = Ok(None);
        let mut arweave = Ok(None);

        Self::visit_url(url, |s| {
            if let Some(c) = Self::try_ipfs(s) {
                Self::advance_heuristic(&mut ipfs, c);
            }

            if let Some(t) = Self::try_arweave(s) {
                Self::advance_heuristic(&mut arweave, t);
            }
        });

        Self {
            ipfs: ipfs.ok().flatten(),
            arweave: arweave.ok().flatten(),
        }
    }
}
