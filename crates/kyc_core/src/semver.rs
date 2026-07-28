pub use semver::{
    Version as SemanticVersion,
    VersionReq as VersionRequirement,
};

pub fn parse_version(s: &str) -> Result<SemanticVersion, String> {
    SemanticVersion::parse(s).map_err(|e| format!("invalid version '{}': {}", s, e))
}

pub fn parse_requirement(s: &str) -> Result<VersionRequirement, String> {
    VersionRequirement::parse(s).map_err(|e| format!("invalid version requirement '{}': {}", s, e))
}

pub fn matches(version: &SemanticVersion, req: &VersionRequirement) -> bool {
    req.matches(version)
}

pub fn version_to_string(v: &SemanticVersion) -> String {
    format!("{}.{}.{}", v.major, v.minor, v.patch)
}

pub fn is_stable(v: &SemanticVersion) -> bool {
    v.major >= 1
}

pub fn is_prerelease(v: &SemanticVersion) -> bool {
    !v.pre.is_empty()
}
