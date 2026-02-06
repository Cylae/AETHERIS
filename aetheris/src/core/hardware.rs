use log::info;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HardwareProfile {
    Low,      // < 4GB RAM, <= 2 cores
    Standard, // 4-16GB RAM
    High,     // > 16GB RAM
}

#[derive(Debug, Clone)]
pub struct HardwareInfo {
    pub profile: HardwareProfile,
    pub ram_gb: u64,
    pub cpu_cores: usize,
    pub has_nvidia: bool,
    pub has_intel_quicksync: bool,
    pub disk_gb: u64,
    pub swap_gb: u64,
    pub user_id: String,
    pub group_id: String,
}

use crate::ports::HardwareSpecs;

impl HardwareInfo {
    pub fn from_specs(specs: HardwareSpecs, user_id: String, group_id: String) -> Self {
        let profile = Self::evaluate_profile(specs.ram_gb, specs.cpu_cores, specs.swap_gb);

        info!("Hardware Specs: RAM={}GB, Swap={}GB, Disk={}GB, Cores={}, Profile={:?}", specs.ram_gb, specs.swap_gb, specs.disk_gb, specs.cpu_cores, profile);
        if specs.has_nvidia { info!("Nvidia GPU Detected"); }
        if specs.has_intel_quicksync { info!("Intel QuickSync Detected"); }
        info!("User Context: UID={}, GID={}", user_id, group_id);

        Self {
            profile,
            ram_gb: specs.ram_gb,
            cpu_cores: specs.cpu_cores,
            has_nvidia: specs.has_nvidia,
            has_intel_quicksync: specs.has_intel_quicksync,
            disk_gb: specs.disk_gb,
            swap_gb: specs.swap_gb,
            user_id,
            group_id,
        }
    }

    // For testing logic without system calls
    pub fn evaluate_profile(ram_gb: u64, cpu_cores: usize, swap_gb: u64) -> HardwareProfile {
        if ram_gb > 16 {
            HardwareProfile::High
        } else if ram_gb < 4 || cpu_cores <= 2 {
            HardwareProfile::Low
        } else {
             // Standard range (4-16GB RAM, >2 Cores)
             // If RAM is on the lower end (4-8GB) and no swap, downgrade to Low for safety
             // (This is a defensive measure to prevent OOM on machines with just enough RAM but no swap buffer)
             if ram_gb < 8 && swap_gb < 1 {
                 HardwareProfile::Low
             } else {
                 HardwareProfile::Standard
             }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_profile_evaluation() {
        assert_eq!(HardwareInfo::evaluate_profile(2, 4, 2), HardwareProfile::Low); // Low RAM
        assert_eq!(HardwareInfo::evaluate_profile(8, 1, 2), HardwareProfile::Low); // Low Cores
        assert_eq!(HardwareInfo::evaluate_profile(32, 8, 0), HardwareProfile::High); // High RAM ignores Swap
        assert_eq!(HardwareInfo::evaluate_profile(8, 4, 0), HardwareProfile::Standard); // 8GB RAM No Swap -> Standard
        assert_eq!(HardwareInfo::evaluate_profile(6, 4, 0), HardwareProfile::Low); // 6GB RAM No Swap -> Low
        assert_eq!(HardwareInfo::evaluate_profile(6, 4, 2), HardwareProfile::Standard); // 6GB RAM + Swap -> Standard
    }

}
