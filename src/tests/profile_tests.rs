#[cfg(test)]
mod tests {
    use crate::models::*;
    use crate::modules::state_manager::InstallationStateManager;
    use crate::modules::config::ConfigManager;
    use std::collections::{HashMap, HashSet};
    
    #[test]
    fn test_profile_creation() {
        let config = ConfigManager::new().unwrap();
        let mut state_mgr = InstallationStateManager::new(config);
        
        state_mgr.create_profile("work", None).unwrap();
        assert!(state_mgr.profiles.contains_key("work"));
        
        state_mgr.create_profile("personal", Some("work".to_string())).unwrap();
        assert!(state_mgr.profiles.contains_key("personal"));
        
        let personal = state_mgr.profiles.get("personal").unwrap();
        assert_eq!(personal.parent, Some("work".to_string()));
    }
    
    #[test]
    fn test_smart_install() {
        let config = ConfigManager::new().unwrap();
        let mut state_mgr = InstallationStateManager::new(config);
        
        state_mgr.create_profile("test", None).unwrap();
        state_mgr.switch_profile("test").unwrap();
        
        // First install
        state_mgr.smart_install("nodejs", InstallScope::Global).unwrap();
        assert!(state_mgr.is_installed("nodejs"));
        assert!(state_mgr.is_active("nodejs"));
        
        // Second install (should just activate)
        state_mgr.create_profile("test2", None).unwrap();
        state_mgr.switch_profile("test2").unwrap();
        state_mgr.smart_install("nodejs", InstallScope::Global).unwrap();
        
        // Check both profiles have it active
        let record = state_mgr.installations.get("nodejs").unwrap();
        assert!(record.active_for.contains("test"));
        assert!(record.active_for.contains("test2"));
    }
    
    #[test]
    fn test_removal_strategies() {
        let config = ConfigManager::new().unwrap();
        let mut state_mgr = InstallationStateManager::new(config);
        
        state_mgr.create_profile("profile1", None).unwrap();
        state_mgr.switch_profile("profile1").unwrap();
        state_mgr.smart_install("package1", InstallScope::Profile).unwrap();
        
        // Deactivate only
        state_mgr.handle_removal("package1", RemovalStrategy::Deactivate).unwrap();
        assert!(state_mgr.is_installed("package1"));
        assert!(!state_mgr.is_active("package1"));
        
        // Reactivate
        state_mgr.activate_for_profile("package1").unwrap();
        
        // Smart remove (should actually uninstall since only one profile uses it)
        state_mgr.handle_removal("package1", RemovalStrategy::SmartRemove).unwrap();
        assert!(!state_mgr.is_installed("package1"));
    }
    
    #[test]
    fn test_os_detection() {
        let os = OsType::detect();
        
        #[cfg(target_os = "macos")]
        assert_eq!(os, OsType::MacOS);
        
        #[cfg(target_os = "windows")]
        assert_eq!(os, OsType::Windows);
        
        #[cfg(target_os = "linux")]
        assert_eq!(os, OsType::Linux);
    }
    
    #[test]
    fn test_environment_state() {
        let mut env_state = EnvironmentState::default();
        
        env_state.paths_prepend.push("/usr/local/bin".to_string());
        env_state.paths_append.push("/opt/bin".to_string());
        env_state.variables.insert("TEST_VAR".to_string(), "test_value".to_string());
        env_state.aliases.insert("ll".to_string(), "ls -la".to_string());
        
        assert!(env_state.active);
        assert_eq!(env_state.paths_prepend.len(), 1);
        assert_eq!(env_state.paths_append.len(), 1);
        assert_eq!(env_state.variables.get("TEST_VAR"), Some(&"test_value".to_string()));
        assert_eq!(env_state.aliases.get("ll"), Some(&"ls -la".to_string()));
    }
    
    #[test]
    fn test_profile_switching_performance() {
        use std::time::Instant;
        
        let config = ConfigManager::new().unwrap();
        let mut state_mgr = InstallationStateManager::new(config);
        
        // Create profiles
        state_mgr.create_profile("profile1", None).unwrap();
        state_mgr.create_profile("profile2", None).unwrap();
        
        // Add some packages
        state_mgr.switch_profile("profile1").unwrap();
        for i in 0..10 {
            state_mgr.smart_install(&format!("package{}", i), InstallScope::Profile).unwrap();
        }
        
        // Measure switching time
        let start = Instant::now();
        state_mgr.switch_profile("profile2").unwrap();
        let duration = start.elapsed();
        
        // Should be very fast (< 100ms for simple state switch)
        assert!(duration.as_millis() < 100, "Profile switch took {:?}", duration);
    }
}