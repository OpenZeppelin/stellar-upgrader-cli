use crate::UpgradeArgs;
use super::{SecurityCheck, SecurityCheckContext};
use std::process::Command;

pub struct VersionCheck;

impl VersionCheck {
    pub fn new() -> Self {
        VersionCheck
    }

    /// Get contract metadata for a given contract ID or WASM hash
    fn get_contract_metadata(&self, args: &UpgradeArgs, wasm_hash: Option<&str>) -> Result<String, String> {
        let command = if let Some(hash) = wasm_hash {
            format!(
                "stellar contract info meta --wasm-hash {} --network {} --output json",
                hash, args.network
            )
        } else {
            format!(
                "stellar contract info meta --id {} --network {} --output json",
                args.id, args.network
            )
        };

        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", &command])
                .output()
        } else {
            Command::new("sh")
                .args(["-c", &command])
                .output()
        };

        match output {
            Ok(output) => {
                if output.status.success() {
                    if let Ok(stdout) = String::from_utf8(output.stdout) {
                        Ok(stdout)
                    } else {
                        Err("Failed to parse metadata output".to_string())
                    }
                } else {
                    if let Ok(stderr) = String::from_utf8(output.stderr) {
                        Err(format!("Failed to get contract metadata: {}", stderr))
                    } else {
                        Err("Failed to get contract metadata".to_string())
                    }
                }
            }
            Err(e) => Err(format!("Failed to execute command: {}", e)),
        }
    }

    /// Extract binver from metadata JSON
    pub fn extract_binver(&self, metadata_json: &str) -> Result<String, String> {
        // Parse the JSON to find binver
        // The format is: [{"sc_meta_v0":{"key":"binver","val":"2.0.0"}}, ...]
        
        // Simple JSON parsing - look for "binver" key
        if let Some(start) = metadata_json.find(r#""key":"binver""#) {
            if let Some(val_start) = metadata_json[start..].find(r#""val":""#) {
                let val_start_pos = start + val_start + 7; // 7 = length of `"val":"`
                if let Some(val_end) = metadata_json[val_start_pos..].find('"') {
                    let version = &metadata_json[val_start_pos..val_start_pos + val_end];
                    return Ok(version.to_string());
                }
            }
        }
        
        Err("binver not found in metadata".to_string())
    }

    /// Compare two semantic versions (e.g., "1.0.0" vs "2.0.0")
    pub fn compare_versions(&self, current: &str, new: &str) -> Result<bool, String> {
        let current_parts: Result<Vec<u32>, _> = current.split('.').map(|s| s.parse()).collect();
        let new_parts: Result<Vec<u32>, _> = new.split('.').map(|s| s.parse()).collect();

        let current_parts = current_parts.map_err(|_| format!("Invalid version format: {}", current))?;
        let new_parts = new_parts.map_err(|_| format!("Invalid version format: {}", new))?;

        // Pad with zeros if one version has fewer parts
        let max_len = current_parts.len().max(new_parts.len());
        let mut current_padded = current_parts;
        let mut new_padded = new_parts;
        
        current_padded.resize(max_len, 0);
        new_padded.resize(max_len, 0);

        // Compare versions part by part
        for i in 0..max_len {
            if new_padded[i] > current_padded[i] {
                return Ok(true);
            } else if new_padded[i] < current_padded[i] {
                return Ok(false);
            }
        }

        // Versions are equal
        Ok(false)
    }
}

impl SecurityCheck for VersionCheck {
    fn name(&self) -> &str {
        "Version Check"
    }

    fn run(&self, args: &UpgradeArgs, _context: &mut SecurityCheckContext) -> Result<(), String> {
        println!("Fetching current contract metadata...");
        let current_metadata = self.get_contract_metadata(args, None)?;
        let current_version = self.extract_binver(&current_metadata)?;

        println!("Fetching new WASM metadata...");
        let new_metadata = self.get_contract_metadata(args, Some(&args.wasm_hash))?;
        let new_version = self.extract_binver(&new_metadata)?;

        println!("Current version: {}", current_version);
        println!("New version: {}", new_version);

        if self.compare_versions(&current_version, &new_version)? {
            println!("✅ New version ({}) is greater than current version ({})", new_version, current_version);
            Ok(())
        } else {
            Err(format!("❌ New version ({}) is not greater than current version ({}). Version downgrades are not recommended.", new_version, current_version))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_binver() {
        let check = VersionCheck::new();
        
        let metadata = r#"[{"sc_meta_v0":{"key":"binver","val":"2.0.0"}},{"sc_meta_v0":{"key":"rsver","val":"1.85.0"}}]"#;
        let version = check.extract_binver(metadata).unwrap();
        assert_eq!(version, "2.0.0");
    }

    #[test]
    fn test_extract_binver_not_found() {
        let check = VersionCheck::new();
        
        let metadata = r#"[{"sc_meta_v0":{"key":"rsver","val":"1.85.0"}}]"#;
        let result = check.extract_binver(metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_compare_versions() {
        let check = VersionCheck::new();
        
        // Test major version upgrade
        assert!(check.compare_versions("1.0.0", "2.0.0").unwrap());
        assert!(!check.compare_versions("2.0.0", "1.0.0").unwrap());
        
        // Test minor version upgrade
        assert!(check.compare_versions("1.0.0", "1.1.0").unwrap());
        assert!(!check.compare_versions("1.1.0", "1.0.0").unwrap());
        
        // Test patch version upgrade
        assert!(check.compare_versions("1.0.0", "1.0.1").unwrap());
        assert!(!check.compare_versions("1.0.1", "1.0.0").unwrap());
        
        // Test equal versions
        assert!(!check.compare_versions("1.0.0", "1.0.0").unwrap());
        
        // Test different length versions
        assert!(check.compare_versions("1.0", "1.0.1").unwrap());
        assert!(check.compare_versions("1", "1.0.1").unwrap());
        assert!(!check.compare_versions("1.0.1", "1.0").unwrap());
    }

    #[test]
    fn test_compare_versions_invalid() {
        let check = VersionCheck::new();
        
        let result = check.compare_versions("invalid", "1.0.0");
        assert!(result.is_err());
        
        let result = check.compare_versions("1.0.0", "invalid");
        assert!(result.is_err());
    }
} 