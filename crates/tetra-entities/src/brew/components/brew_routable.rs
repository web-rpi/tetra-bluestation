use tetra_config::SharedConfig;

use crate::brew::worker::BrewConfig;

pub fn is_brew_routable(config: &SharedConfig, brew_config: &BrewConfig, ssi: u32) -> bool {
    if ssi <= 90 {
        // Brew doesn't route 0..=90
        return false;
    }
    if config.config().cell.local_ssi_ranges.contains(ssi) {
        // Range overridden as local
        return false;
    }

    // We can either have whitelist or blacklist, not both. Check if any one present, then use that
    // If none present, default to allow

    if let Some(whitelist) = &brew_config.whitelisted_ssi_ranges {
        if whitelist.contains(ssi) {
            // Range explicitly whitelisted for routing to Brew
            return true;
        } else {
            // Not in whitelist - block routing to Brew
            return false;
        }
    }

    if let Some(blacklist) = &brew_config.blacklisted_ssi_ranges {
        if blacklist.contains(ssi) {
            // Range explicitly blacklisted from routing to Brew
            return false;
        } else {
            // Not in blacklist - allow routing to Brew
            return true;
        }
    }

    // No whitelist or blacklist present, default to allow
    true
}
