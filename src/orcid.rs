use std::str::FromStr;

use serde::Serialize;

const ORCID_BASE: &str = "https://orcid.org/";
const ORCID_BASE_HTTP: &str = "http://orcid.org/";

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy, Default)]
pub enum Format {
    Short,
    Hyphen,
    #[default]
    Url,
}

#[derive(Debug)]
pub struct Formatted {
    format: Format,
    orcid: OrcId,
}

fn trim_base_url(s: &str) -> &str {
    for base in [ORCID_BASE, ORCID_BASE_HTTP] {
        if s.starts_with(base) {
            return &s[base.len()..];
        }
    }
    s
}

impl std::fmt::Display for Formatted {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.format {
            Format::Short => {
                for c in self.orcid.iter() {
                    write!(f, "{c}")?;
                }
            }
            Format::Hyphen => {
                for (idx, c) in self.orcid.iter().enumerate() {
                    if idx > 0 && idx % 4 == 0 {
                        write!(f, "-")?;
                    }
                    write!(f, "{c}")?;
                }
            }
            Format::Url => self.orcid.fmt(f)?,
        }
        Ok(())
    }
}

#[derive(PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy)]
pub struct OrcId {
    /// All values must be in 0..=9
    id: [u8; 15],
    /// Checksum value, in 0..=10
    checksum: u8,
}

impl std::fmt::Debug for OrcId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OrcId").field("url", &self.full()).finish()
    }
}

impl OrcId {
    /// Can fail if any of the digits are >9.
    pub fn try_new(id: [u8; 15]) -> Result<Self, String> {
        let checksum = Self::calc_checksum(&id)?;
        Ok(Self { id, checksum })
    }

    pub fn full(&self) -> Formatted {
        Formatted {
            format: Format::Url,
            orcid: *self,
        }
    }

    pub fn short(&self) -> Formatted {
        Formatted {
            format: Format::Short,
            orcid: *self,
        }
    }

    pub fn id_only(&self) -> Formatted {
        Formatted {
            format: Format::Hyphen,
            orcid: *self,
        }
    }

    fn try_new_checked(id: [u8; 15], checksum: u8) -> Result<Self, String> {
        let expected = Self::calc_checksum(&id)?;
        if expected != checksum {
            return Err(format!(
                "Invalid checksum: expected {}, got {}",
                expected, checksum
            ));
        }
        Ok(Self { id, checksum })
    }

    /// Fails if any of the digits are >9.
    fn calc_checksum(arr: &[u8; 15]) -> Result<u8, String> {
        let mut total = 0;
        for n in arr.iter() {
            if *n > 9 {
                return Err(format!("Invalid ORCID digit: {}", n));
            }
            total = (total + *n as u32) * 2;
        }
        let total = arr.iter().fold(0u32, |total, n| (total + *n as u32) * 2);
        let remainder = total % 11;
        Ok(((12 - remainder) % 11) as u8)
    }

    /// Iterate through meaningful characters of the ORCID (digits and checksum).
    fn iter(&self) -> impl Iterator<Item = char> + '_ {
        OrcIdDigits {
            orcid: self,
            idx: 0,
        }
    }
}

struct OrcIdDigits<'a> {
    orcid: &'a OrcId,
    idx: usize,
}

impl Iterator for OrcIdDigits<'_> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        match self.idx {
            0..15 => {
                let c = std::char::from_digit(self.orcid.id[self.idx] as u32, 10).unwrap();
                self.idx += 1;
                Some(c)
            }
            15 => {
                let c = if self.orcid.checksum == 10 {
                    'X'
                } else {
                    std::char::from_digit(self.orcid.checksum as u32, 10).unwrap()
                };
                self.idx += 1;
                Some(c)
            }
            _ => None,
        }
    }
}

impl FromStr for OrcId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut arr = [0u8; 15];
        let mut out = None;
        for (idx, c) in trim_base_url(s).chars().filter(|c| *c != '-').enumerate() {
            out = match idx {
                0..=14 => {
                    let Some(d) = c.to_digit(10) else {
                        return Err(format!("Invalid character '{}' in ORCID", c));
                    };
                    arr[idx] = d as u8;
                    continue;
                }
                15 => {
                    let chk = if c == 'X' {
                        10
                    } else {
                        let Some(d) = c.to_digit(10) else {
                            return Err(format!("Invalid character '{}' in ORCID", c));
                        };
                        d as u8
                    };
                    Some(Self::try_new_checked(arr, chk)?)
                }
                _ => return Err("ORCID too long".to_string()),
            };
        }

        if let Some(o) = out {
            Ok(o)
        } else {
            Err("ORCID too short".into())
        }
    }
}

impl std::fmt::Display for OrcId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(ORCID_BASE)?;
        for (idx, c) in self.iter().enumerate() {
            if idx > 0 && idx % 4 == 0 {
                f.write_str("-")?;
            }
            write!(f, "{c}")?;
        }
        Ok(())
    }
}

impl Serialize for OrcId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for OrcId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        OrcId::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl From<Formatted> for OrcId {
    fn from(value: Formatted) -> Self {
        value.orcid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_orcids() {
        for s in [
            // non-HTTP(S) URL scheme
            "file://orcid.org/0000-0002-1296-7310",
            // allowed under ISO standard but not used for ORCiD
            "0000 0002 1296 7310",
            // invalid checksum digit
            "0000-0002-1296-7311",
            // too long
            "0000-0002-1296-73109",
            // too short
            "0000-0002-1296-731",
        ] {
            println!("{s}");
            OrcId::from_str(s).unwrap_err();
            let json = format!("\"{s}\"");
            serde_json::from_str::<OrcId>(&json).unwrap_err();
        }
    }

    #[test]
    fn test_valid_orcids() {
        for s in [
            // HTTPS form
            "https://orcid.org/0000-0002-1296-7310",
            // HTTP form (website redirects)
            "http://orcid.org/0000-0002-1296-7310",
            // hyphenated form
            "0000-0002-1296-7310",
            // short form as URL (website redirects)
            "https://orcid.org/0000000212967310",
            // short form
            "0000000212967310",
        ] {
            let val = OrcId::from_str(s).unwrap();
            let json = format!("\"{s}\"");
            serde_json::from_str::<OrcId>(&json).unwrap();
            let s2 = val.to_string();
            assert!(s2.starts_with(ORCID_BASE));
        }
    }
}
