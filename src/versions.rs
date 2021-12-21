use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub repo: String,
    pub path: String,
    pub created: String,
    pub created_by: String,
    pub last_modified: String,
    pub modified_by: String,
    pub last_updated: String,
    pub children: Vec<Child>,
    pub uri: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Child {
    pub uri: String,
    pub folder: bool,
}

#[derive(Default, PartialEq)]
pub struct SemVer {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
    pub prerelease: Option<u16>,
}

impl PartialOrd for SemVer {
    fn partial_cmp(&self, other: &SemVer) -> Option<Ordering> {
        if self.major != other.major {
            return Some(self.major.cmp(&other.major));
        }
        if self.minor != other.minor {
            return Some(self.minor.cmp(&other.minor));
        }
        if self.patch != other.patch {
            return Some(self.patch.cmp(&other.patch));
        }
        if self.prerelease.is_some() && other.prerelease.is_some() {
            return Some(self.prerelease.cmp(&other.prerelease));
        }
        if self.prerelease.is_some() {
            return Some(Ordering::Less);
        }
        if other.prerelease.is_some() {
            return Some(Ordering::Greater);
        }
        Some(Ordering::Equal)
    }
}

pub fn parse_semver(semver: &str) -> Option<SemVer> {
    let parts: Vec<Option<u16>> = semver
        .split("-")
        .map(|x| x.parse::<u16>().ok())
        .take(4)
        .collect();

    match parts[..] {
        [Some(major), Some(minor), Some(patch), ..] => {
            let mut result = SemVer {
                major,
                minor,
                patch,
                prerelease: None,
            };

            if parts.len() == 4 {
                result.prerelease = parts[3];
            }

            Some(result)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::versions;
    use crate::versions::parse_semver;
    use std::cmp::Ordering;

    fn make_semver(
        major: u16,
        minor: u16,
        patch: u16,
        prerelease: Option<u16>,
    ) -> versions::SemVer {
        versions::SemVer {
            major,
            minor,
            patch,
            prerelease,
        }
    }

    #[test]
    fn parse() {
        assert_eq!(parse_semver("1-2-3-4").unwrap().prerelease, Some(4));
        assert_eq!(parse_semver("1-2-3").unwrap().prerelease, None);
        assert_eq!(parse_semver("1-2-3-4-5").unwrap().prerelease, Some(4));
        assert_eq!(parse_semver("1-2").is_none(), true);
        assert_eq!(parse_semver("1-2-l").is_none(), true);
    }

    #[test]
    fn major() {
        let a = make_semver(1, 2, 3, None);
        let b = make_semver(2, 2, 3, None);
        assert_eq!(a.partial_cmp(&b), Some(Ordering::Less));
    }

    #[test]
    fn minor() {
        let a = make_semver(1, 3, 3, None);
        let b = make_semver(1, 2, 3, None);
        assert_eq!(a.partial_cmp(&b), Some(Ordering::Greater));
    }

    #[test]
    fn patch() {
        let a = make_semver(1, 2, 3, None);
        let b = make_semver(1, 2, 4, None);
        assert_eq!(a.partial_cmp(&b), Some(Ordering::Less));
    }

    #[test]
    fn same() {
        let a = make_semver(1, 2, 3, None);
        let b = make_semver(1, 2, 3, None);
        assert_eq!(a.partial_cmp(&b), Some(Ordering::Equal));
    }

    #[test]
    fn prerelease() {
        let mut a = make_semver(1, 2, 3, Some(9));
        let mut b = make_semver(1, 2, 3, None);
        assert_eq!(a.partial_cmp(&b), Some(Ordering::Less));
        a = make_semver(1, 2, 3, Some(8));
        b = make_semver(1, 2, 3, Some(9));
        assert_eq!(a.partial_cmp(&b), Some(Ordering::Less));
        a = make_semver(1, 2, 3, Some(9));
        b = make_semver(1, 2, 3, Some(9));
        assert_eq!(a.partial_cmp(&b), Some(Ordering::Equal));
    }
}

pub fn do_stuff() -> Result<()> {
    // TODO
    // let response = reqwest::blocking::get("https://artifactory.it.keysight.com/artifactory/generic-local-pwsg/siggen/packages-linux/develop/siggen_1-9-1-9_2021-11-22_linux.zip")?;
    // let response = reqwest::blocking::get("https://artifactory.it.keysight.com/artifactory/generic-local-pwsg/siggen/packages-linux/develop/")?;
    let generic_local_pwsg =
        "https://artifactory.it.keysight.com/artifactory/api/storage/generic-local-pwsg/siggen";
    let response =
        reqwest::blocking::get(format!("{}/packages-linux/develop", generic_local_pwsg))?;
    let temp: Root = serde_json::from_str(&response.text()?)?;
    dbg!(&temp);

    for child in temp.children {
        println!("{}", child.uri);
    }
    // let bytes = response.bytes()?;
    // let mut out = File::create("/home/bhutch/projects/siggen_toolkit/temp.zip")?;
    // std::io::copy(&mut bytes.as_ref(), &mut out)?;

    Ok(())
}

pub fn get() -> Root {
    let generic_local_pwsg =
        "https://artifactory.it.keysight.com/artifactory/api/storage/generic-local-pwsg/siggen";
    match reqwest::blocking::get(format!("{}/packages-linux/develop", generic_local_pwsg)) {
        Ok(response) => {
            serde_json::from_str(&response.text().unwrap_or_default()).unwrap_or_default()
        }
        _ => Root::default(),
    }
}
