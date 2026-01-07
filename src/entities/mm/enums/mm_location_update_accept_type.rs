/// Clause 16.10.35a Location update accept type
/// Bits: 3
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MmLocationUpdateAcceptType {
    RoamingLocationUpdating = 0,
    TemporaryRegistration = 1,
    PeriodicLocationUpdating = 2,
    ItsiAttach = 3,
    ServiceRestorationRoamingLocationUpdating = 4,
    MigratingOrServiceRestorationMigratingLocationUpdating = 5,
    DemandLocationUpdating = 6,
    DisabledMsUpdating = 7,
}

impl std::convert::TryFrom<u64> for MmLocationUpdateAcceptType {
    type Error = ();
    fn try_from(x: u64) -> Result<Self, Self::Error> {
        match x {
            0 => Ok(MmLocationUpdateAcceptType::RoamingLocationUpdating),
            1 => Ok(MmLocationUpdateAcceptType::TemporaryRegistration),
            2 => Ok(MmLocationUpdateAcceptType::PeriodicLocationUpdating),
            3 => Ok(MmLocationUpdateAcceptType::ItsiAttach),
            4 => Ok(MmLocationUpdateAcceptType::ServiceRestorationRoamingLocationUpdating),
            5 => Ok(MmLocationUpdateAcceptType::MigratingOrServiceRestorationMigratingLocationUpdating),
            6 => Ok(MmLocationUpdateAcceptType::DemandLocationUpdating),
            7 => Ok(MmLocationUpdateAcceptType::DisabledMsUpdating),
            _ => Err(()),
        }
    }
}

impl MmLocationUpdateAcceptType {
    /// Convert this enum back into the raw integer value
    pub fn into_raw(self) -> u64 {
        match self {
            MmLocationUpdateAcceptType::RoamingLocationUpdating => 0,
            MmLocationUpdateAcceptType::TemporaryRegistration => 1,
            MmLocationUpdateAcceptType::PeriodicLocationUpdating => 2,
            MmLocationUpdateAcceptType::ItsiAttach => 3,
            MmLocationUpdateAcceptType::ServiceRestorationRoamingLocationUpdating => 4,
            MmLocationUpdateAcceptType::MigratingOrServiceRestorationMigratingLocationUpdating => 5,
            MmLocationUpdateAcceptType::DemandLocationUpdating => 6,
            MmLocationUpdateAcceptType::DisabledMsUpdating => 7,
        }
    }
}

impl From<MmLocationUpdateAcceptType> for u64 {
    fn from(e: MmLocationUpdateAcceptType) -> Self { e.into_raw() }
}

impl core::fmt::Display for MmLocationUpdateAcceptType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            MmLocationUpdateAcceptType::RoamingLocationUpdating => write!(f, "RoamingLocationUpdating"),
            MmLocationUpdateAcceptType::TemporaryRegistration => write!(f, "TemporaryRegistration"),
            MmLocationUpdateAcceptType::PeriodicLocationUpdating => write!(f, "PeriodicLocationUpdating"),
            MmLocationUpdateAcceptType::ItsiAttach => write!(f, "ItsiAttach"),
            MmLocationUpdateAcceptType::ServiceRestorationRoamingLocationUpdating => write!(f, "ServiceRestorationRoamingLocationUpdating"),
            MmLocationUpdateAcceptType::MigratingOrServiceRestorationMigratingLocationUpdating => write!(f, "MigratingOrServiceRestorationMigratingLocationUpdating"),
            MmLocationUpdateAcceptType::DemandLocationUpdating => write!(f, "DemandLocationUpdating"),
            MmLocationUpdateAcceptType::DisabledMsUpdating => write!(f, "DisabledMsUpdating"),
        }
    }
}
