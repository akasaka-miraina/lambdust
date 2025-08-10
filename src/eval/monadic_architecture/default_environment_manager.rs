//! Default environment manager implementation

use super::environment_manager_configuration::EnvironmentManagerConfiguration;

/// Default environment manager implementation
#[derive(Debug)]
pub struct DefaultEnvironmentManager {
    /// Configuration
    pub config: EnvironmentManagerConfiguration,
}