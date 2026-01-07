/// Clause 16.10.35 Location update type
/// Almost identical to MmLocationUpdateAcceptType
/// Bits: 3
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MmLocationUpdateType {
    RoamingLocationUpdating = 0,
    TemporaryRegistration = 1,
    PeriodicLocationUpdating = 2,
    ItsiAttach = 3,
    ServiceRestorationRoamingLocationUpdating = 4,
    ServiceRestorationMigratingLocationUpdating = 5,
    DemandLocationUpdating = 6,
    DisabledMsUpdating = 7,
}

impl std::convert::TryFrom<u64> for MmLocationUpdateType {
    type Error = ();
    fn try_from(x: u64) -> Result<Self, Self::Error> {
        match x {
            0 => Ok(MmLocationUpdateType::RoamingLocationUpdating),
            1 => Ok(MmLocationUpdateType::TemporaryRegistration),
            2 => Ok(MmLocationUpdateType::PeriodicLocationUpdating),
            3 => Ok(MmLocationUpdateType::ItsiAttach),
            4 => Ok(MmLocationUpdateType::ServiceRestorationRoamingLocationUpdating),
            5 => Ok(MmLocationUpdateType::ServiceRestorationMigratingLocationUpdating),
            6 => Ok(MmLocationUpdateType::DemandLocationUpdating),
            7 => Ok(MmLocationUpdateType::DisabledMsUpdating),
            _ => Err(()),
        }
    }
}

impl MmLocationUpdateType {
    /// Convert this enum back into the raw integer value
    pub fn into_raw(self) -> u64 {
        match self {
            MmLocationUpdateType::RoamingLocationUpdating => 0,
            MmLocationUpdateType::TemporaryRegistration => 1,
            MmLocationUpdateType::PeriodicLocationUpdating => 2,
            MmLocationUpdateType::ItsiAttach => 3,
            MmLocationUpdateType::ServiceRestorationRoamingLocationUpdating => 4,
            MmLocationUpdateType::ServiceRestorationMigratingLocationUpdating => 5,
            MmLocationUpdateType::DemandLocationUpdating => 6,
            MmLocationUpdateType::DisabledMsUpdating => 7,
        }
    }
}

impl From<MmLocationUpdateType> for u64 {
    fn from(e: MmLocationUpdateType) -> Self { e.into_raw() }
}

impl core::fmt::Display for MmLocationUpdateType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            MmLocationUpdateType::RoamingLocationUpdating => write!(f, "RoamingLocationUpdating"),
            MmLocationUpdateType::TemporaryRegistration => write!(f, "TemporaryRegistration"),
            MmLocationUpdateType::PeriodicLocationUpdating => write!(f, "PeriodicLocationUpdating"),
            MmLocationUpdateType::ItsiAttach => write!(f, "ItsiAttach"),
            MmLocationUpdateType::ServiceRestorationRoamingLocationUpdating => write!(f, "ServiceRestorationRoamingLocationUpdating"),
            MmLocationUpdateType::ServiceRestorationMigratingLocationUpdating => write!(f, "ServiceRestorationMigratingLocationUpdating"),
            MmLocationUpdateType::DemandLocationUpdating => write!(f, "DemandLocationUpdating"),
            MmLocationUpdateType::DisabledMsUpdating => write!(f, "DisabledMsUpdating"),
        }
    }
}
