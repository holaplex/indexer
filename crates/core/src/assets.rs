//! ``AssetIdentifier`` utils - Parse and capture tx and cid
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

/// Supported width sizes for asset proxy
#[derive(Debug, Clone, Copy, strum::FromRepr)]
#[repr(i32)]
pub enum ImageSize {
    /// image natural size
    Original = 0,
    /// tiny image
    Tiny = 100,
    /// extra small image
    XSmall = 400,
    /// small image
    Small = 600,
    /// medium image
    Medium = 800,
    /// large image
    Large = 1400,
}

impl From<i32> for ImageSize {
    fn from(value: i32) -> Self {
        Self::from_repr(value).unwrap_or(Self::Original)
    }
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

    /// parse cid from url
    #[must_use]
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
