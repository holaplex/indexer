//! ``AssetIdentifier`` utils - Parse and capture tx and cid
use cid::Cid;
use url::Url;

/// An Arweave transaction ID
#[derive(Debug, Clone, Copy)]
pub struct ArTxid(pub [u8; 32]);

/// Struct to hold tx ids
#[derive(Debug, Clone)]
pub struct AssetIdentifier {
    /// ipfs cid
    pub ipfs: Option<(Cid, String)>,
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
    fn visit_url(url: &Url, mut f: impl FnMut(&str, Option<usize>)) {
        Some(url.scheme())
            .into_iter()
            .chain(url.domain().into_iter().flat_map(|s| s.split('.')))
            .chain(Some(url.username()))
            .chain(url.password())
            .map(|s| (s, Some(0)))
            .chain(Some((url.path(), None)))
            .chain(
                url.path_segments()
                    .into_iter()
                    .flat_map(|s| s.into_iter().enumerate().map(|(i, s)| (s, Some(i + 1)))),
            )
            .chain(url.query().map(|q| (q, Some(0))))
            .chain(url.fragment().map(|f| (f, Some(0))))
            .for_each(|(s, i)| f(s, i));

        url.query_pairs().for_each(|(k, v)| {
            f(k.as_ref(), Some(0));
            f(v.as_ref(), Some(0));
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
            // We found a match
            Ok(None) => *state = Ok(Some(value)),
            // We found two matches, convert to error due to ambiguity
            Ok(Some(_)) => *state = Err(()),
            Err(()) => (),
        }
    }

    /// parse cid from url
    #[must_use]
    pub fn new(url: &Url) -> Self {
        let mut ipfs = Ok(None);
        let mut arweave = Ok(None);

        Self::visit_url(url, |s, i| {
            if let Some(c) = Self::try_ipfs(s) {
                let path = i
                    .and_then(|i| url.path_segments().map(|s| (i, s)))
                    .map_or_else(String::new, |(i, s)| {
                        s.skip(i)
                            .flat_map(|s| Some("/").into_iter().chain(Some(s)))
                            .collect()
                    });

                Self::advance_heuristic(&mut ipfs, (c, path));
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
