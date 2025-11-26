use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, str::FromStr};
use url::Url;

const SCHEME: &str = "doi:";
const BASE_URL: &str = "https://doi.org/";

/// Normalised to 'prefix/suffix' form, upper case.
#[derive(Debug, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
#[serde(transparent)]
pub struct Doi(String);

impl std::fmt::Display for Doi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for Doi {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url = Url::parse(s).map_err(|e| e.to_string())?;

        let Some(segs) = url.path_segments() else {
            return Err(format!("No URL path segments in DOI '{s}'"));
        };
        let mut pref_suff = segs.fold(VecDeque::with_capacity(2), |mut v, seg| {
            if v.len() >= 2 {
                v.pop_front();
            }
            v.push_back(seg);
            v
        });
        let Some(prefix) = pref_suff.pop_front().filter(|p| !p.is_empty()) else {
            return Err(format!("No prefix/suffix in DOI '{s}'"));
        };
        let mut out = String::new();

        if !prefix.starts_with("10.")
            || prefix.len() < 7
            || prefix.chars().any(|c| c != '.' && !c.is_ascii_digit())
        {
            for c in prefix.chars() {
                if c != '.' && !c.is_ascii_digit() {
                    return Err(format!("Invalid DOI prefix '{prefix}'"));
                }
                out.extend(c.to_uppercase());
            }
        }

        out.push('/');
        let Some(suffix) = pref_suff.pop_front().filter(|p| !p.is_empty()) else {
            return Err(format!("No prefix/suffix in DOI '{s}'"));
        };
        out.extend(suffix.chars().flat_map(|c| c.to_uppercase()));

        // if let Some(q) = url.query() {
        //     out.push('?');
        //     out.push_str(q);
        // }
        // if let Some(f) = url.fragment() {
        //     out.push('#');
        //     out.push_str(f);
        // }
        Ok(Self(out))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Format {
    /// Print a URI with the scheme `doi:` followed by the DOI name.
    ///
    /// Preferred by the DOI Handbook.
    Scheme,
    /// Print a URL using the https://doi.org/ proxy.
    ///
    /// Preferred by APA, DataCite etc.
    DoiOrg,
    /// Print only the DOI name; should only be used when the context makes it clear that it is a DOI.
    Name,
}

/// Wrapper over a reference to a DOI and a way to format it.
pub struct Formatted<'a> {
    format: Format,
    doi: &'a Doi,
}

impl<'a> std::fmt::Display for Formatted<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.format {
            Format::Scheme => f.write_str(SCHEME)?,
            Format::DoiOrg => f.write_str(BASE_URL)?,
            Format::Name => (),
        }
        f.write_str(&self.doi.0)
    }
}

impl<'de> Deserialize<'de> for Doi {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let d: Doi = s.parse().map_err(serde::de::Error::custom)?;
        Ok(d)
    }
}
