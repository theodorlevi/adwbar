use std::fs;

/// Reads current CPU usage from /proc/stat
pub fn read_cpu_usage() -> String {
    match fs::read_to_string("/proc/stat") {
        Ok(content) => {
            if let Some(line) = content.lines().next() {
                let values: Vec<&str> = line.split_whitespace().collect();
                if values.len() > 4 {
                    if let (Ok(user), Ok(nice), Ok(system), Ok(idle)) = (
                        values[1].parse::<u64>(),
                        values[2].parse::<u64>(),
                        values[3].parse::<u64>(),
                        values[4].parse::<u64>(),
                    ) {
                        let total = user + nice + system + idle;
                        let usage = if total > 0 {
                            ((total - idle) * 100) / total
                        } else {
                            0
                        };
                        return format!("{}%", usage);
                    }
                }
            }
            "N/A".to_string()
        }
        Err(_) => "N/A".to_string(),
    }
}

/// Reads AMD GPU usage from sysfs
pub fn read_gpu_usage() -> String {
    match fs::read_to_string("/sys/class/drm/card0/device/gpu_busy_percent") {
        Ok(content) => {
            if let Ok(usage) = content.trim().parse::<u32>() {
                return format!("{}%", usage);
            }
            "N/A".to_string()
        }
        Err(_) => "N/A".to_string(),
    }
}
