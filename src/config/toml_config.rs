use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, BufReader};
use std::path::Path;

use serde::Deserialize;
use toml::Value;

use crate::config::stack_config::{CfgPhyIo, PhyBackend};
use crate::{
    CfgCellInfo, CfgNetInfo, SharedConfig, StackConfig, StackMode, StackState,
};
use super::stack_config_soapy::{CfgSoapySdr, LimeSdrCfg, SXceiverCfg, UsrpB2xxCfg};

/// Build `SharedConfig` from a TOML configuration file
pub fn from_toml_str(toml_str: &str) -> Result<SharedConfig, Box<dyn std::error::Error>> {
    let root: TomlConfigRoot = toml::from_str(toml_str)?;

    // Various sanity checks
    if !root.config_version.eq("0.4") {
        tracing::warn!("Unrecognized config_version: {}", root.config_version);
    }
    if !root.extra.is_empty() {
        tracing::warn!("Unrecognized top-level fields: {:?}", sorted_keys(&root.extra));
    }
    if let Some(ref phy) = root.phy_io {
        if !phy.extra.is_empty() {
            tracing::warn!("Unrecognized fields in phy_io: {:?}", sorted_keys(&phy.extra));
        }
        if let Some(ref soapy) = phy.soapysdr {
            if !soapy.extra.is_empty() {
                tracing::warn!("Unrecognized fields in phy_io.soapysdr: {:?}", sorted_keys(&soapy.extra));
            }
        }
    }
    if let Some(ref ni) = root.net_info {
        if !ni.extra.is_empty() {
            tracing::warn!("Unrecognized fields in net_info: {:?}", sorted_keys(&ni.extra));
        }
    }
    if let Some(ref ci) = root.cell_info {
        if !ci.extra.is_empty() {
            tracing::warn!("Unrecognized fields in cell_info: {:?}", sorted_keys(&ci.extra));
        }
    }
    if let Some(ref ss) = root.stack_state {
        if !ss.extra.is_empty() {
            tracing::warn!("Unrecognized fields in stack_state: {:?}", sorted_keys(&ss.extra));
        }
    }

    // Require net_info to be explicitly set
    let Some(net_info_dto) = root.net_info else {
        return Err("net_info section is required in config file".into());
    };
    let Some(mcc) = net_info_dto.mcc else {
        return Err("net_info.mcc is required in config file".into());
    };
    let Some(mnc) = net_info_dto.mnc else {
        return Err("net_info.mnc is required in config file".into());
    };

    // Build config from required and optional values
    let mut cfg = StackConfig {
        stack_mode: root.stack_mode,
        phy_io: CfgPhyIo::default(),
        net: CfgNetInfo { mcc, mnc },
        cell: CfgCellInfo::default(),
    };

    // Handle new phy_io structure
    if let Some(phy) = root.phy_io {
        apply_phy_io_patch(&mut cfg.phy_io, phy);
    }

    if let Some(ci) = root.cell_info {
        apply_cell_info_patch(&mut cfg.cell, ci);
    }

    // Mutable runtime state. Currently just a placeholder and not yet actually used
    let mut state = StackState::default();
    if let Some(ss) = root.stack_state {
        if let Some(v) = ss.cell_load_ca {
            state.cell_load_ca = v;
        }
    }

    Ok(SharedConfig::from_parts(cfg, state))
}

/// Build `SharedConfig` from any reader.
pub fn from_reader<R: Read>(reader: R) -> Result<SharedConfig, Box<dyn std::error::Error>> {
    let mut contents = String::new();
    let mut reader = BufReader::new(reader);
    reader.read_to_string(&mut contents)?;
    
    from_toml_str(&contents)
}

/// Build `SharedConfig` from a file path.
pub fn from_file<P: AsRef<Path>>(path: P) -> Result<SharedConfig, Box<dyn std::error::Error>> {
    let f = File::open(path)?;
    let r = BufReader::new(f);
    let cfg = from_reader(r)?;
    Ok(cfg)
}

fn apply_phy_io_patch(dst: &mut CfgPhyIo, src: PhyIoDto) {
    dst.backend = src.backend;
    dst.input_file = src.input_file;
    
    if let Some(soapy_dto) = src.soapysdr {
        let mut soapy_cfg = CfgSoapySdr::default();
        soapy_cfg.ul_freq = soapy_dto.rx_freq;
        soapy_cfg.dl_freq = soapy_dto.tx_freq;
        soapy_cfg.ppm_err = soapy_dto.ppm_err;
        
        // Apply hardware-specific configurations
        if let Some(usrp_dto) = soapy_dto.iocfg_usrpb2xx {
            soapy_cfg.io_cfg.iocfg_usrpb2xx = Some(UsrpB2xxCfg {
                rx_ant: usrp_dto.rx_ant,
                tx_ant: usrp_dto.tx_ant,
                rx_gain_pga: usrp_dto.rx_gain_pga,
                tx_gain_pga: usrp_dto.tx_gain_pga,
            });
        }
        
        if let Some(lime_dto) = soapy_dto.iocfg_limesdr {
            soapy_cfg.io_cfg.iocfg_limesdr = Some(LimeSdrCfg {
                rx_ant: lime_dto.rx_ant,
                tx_ant: lime_dto.tx_ant,
                rx_gain_lna: lime_dto.rx_gain_lna,
                rx_gain_tia: lime_dto.rx_gain_tia,
                rx_gain_pga: lime_dto.rx_gain_pga,
                tx_gain_pad: lime_dto.tx_gain_pad,
                tx_gain_iamp: lime_dto.tx_gain_iamp,
            });
        }
        
        if let Some(sx_dto) = soapy_dto.iocfg_sxceiver {
            soapy_cfg.io_cfg.iocfg_sxceiver = Some(SXceiverCfg {
                rx_ant: sx_dto.rx_ant,
                tx_ant: sx_dto.tx_ant,
                rx_gain_lna: sx_dto.rx_gain_lna,
                rx_gain_pga: sx_dto.rx_gain_pga,
                tx_gain_dac: sx_dto.tx_gain_dac,
                tx_gain_mixer: sx_dto.tx_gain_mixer,
            });
        }
        
        dst.soapysdr = Some(soapy_cfg);
    }
}

fn apply_cell_info_patch(dst: &mut CfgCellInfo, ci: CellInfoDto) {

    dst.main_carrier = ci.main_carrier;
    dst.freq_band = ci.freq_band;
    dst.freq_offset_hz = ci.freq_offset;
    dst.duplex_spacing_id = ci.duplex_spacing;
    dst.reverse_operation = ci.reverse_operation;
    
    // Option
    dst.custom_duplex_spacing = ci.custom_duplex_spacing;

    dst.location_area = ci.location_area;

    if let Some(v) = ci.neighbor_cell_broadcast {
        dst.neighbor_cell_broadcast = v;
    }
    if let Some(v) = ci.cell_load_ca {
        dst.cell_load_ca = v;
    }
    if let Some(v) = ci.late_entry_supported {
        dst.late_entry_supported = v;
    }
    if let Some(v) = ci.subscriber_class {
        dst.subscriber_class = v;
    }
    if let Some(v) = ci.registration {
        dst.registration = v;
    }
    if let Some(v) = ci.deregistration {
        dst.deregistration = v;
    }
    if let Some(v) = ci.priority_cell {
        dst.priority_cell = v;
    }
    if let Some(v) = ci.no_minimum_mode {
        dst.no_minimum_mode = v;
    }
    if let Some(v) = ci.migration {
        dst.migration = v;
    }
    if let Some(v) = ci.system_wide_services {
        dst.system_wide_services = v;
    }
    if let Some(v) = ci.voice_service {
        dst.voice_service = v;
    }
    if let Some(v) = ci.circuit_mode_data_service {
        dst.circuit_mode_data_service = v;
    }
    if let Some(v) = ci.sndcp_service {
        dst.sndcp_service = v;
    }
    if let Some(v) = ci.aie_service {
        dst.aie_service = v;
    }
    if let Some(v) = ci.advanced_link {
        dst.advanced_link = v;
    }
    if let Some(v) = ci.system_code {
        dst.system_code = v;
    }
    if let Some(v) = ci.colour_code {
        dst.colour_code = v;
    }
    if let Some(v) = ci.sharing_mode {
        dst.sharing_mode = v;
    }
    if let Some(v) = ci.ts_reserved_frames {
        dst.ts_reserved_frames = v;
    }
    if let Some(v) = ci.u_plane_dtx {
        dst.u_plane_dtx = v;
    }
    if let Some(v) = ci.frame_18_ext {
        dst.frame_18_ext = v;
    }
}

fn sorted_keys(map: &HashMap<String, Value>) -> Vec<&str> {
    let mut v: Vec<&str> = map.keys().map(|s| s.as_str()).collect();
    v.sort_unstable();
    v
}

/// ----------------------- DTOs for input shape -----------------------

#[derive(Deserialize)]
struct TomlConfigRoot {
    config_version: String,
    stack_mode: StackMode,
    
    // New phy_io structure
    #[serde(default)]
    phy_io: Option<PhyIoDto>,
    
    #[serde(default)]
    net_info: Option<NetInfoDto>,

    #[serde(default)]
    cell_info: Option<CellInfoDto>,

    #[serde(default)]
    stack_state: Option<StackStatePatch>,

    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[derive(Deserialize)]
struct PhyIoDto {
    pub backend: PhyBackend,
    
    #[serde(default)]
    pub input_file: Option<String>,
    
    #[serde(default)]
    pub soapysdr: Option<SoapySdrDto>,

    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[derive(Deserialize)]
struct SoapySdrDto {
    pub rx_freq: f64,
    pub tx_freq: f64,
    pub ppm_err: Option<f64>,
    
    #[serde(default)]
    pub iocfg_usrpb2xx: Option<UsrpB2xxDto>,
    
    #[serde(default)]
    pub iocfg_limesdr: Option<LimeSdrDto>,
    
    #[serde(default)]
    pub iocfg_sxceiver: Option<SXceiverDto>,

    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[derive(Deserialize)]
struct UsrpB2xxDto {
    pub rx_ant: Option<String>,
    pub tx_ant: Option<String>,
    pub rx_gain_pga: Option<f64>,
    pub tx_gain_pga: Option<f64>,
}

#[derive(Deserialize)]
struct LimeSdrDto {
    pub rx_ant: Option<String>,
    pub tx_ant: Option<String>,
    pub rx_gain_lna: Option<f64>,
    pub rx_gain_tia: Option<f64>,
    pub rx_gain_pga: Option<f64>,
    pub tx_gain_pad: Option<f64>,
    pub tx_gain_iamp: Option<f64>,
}

#[derive(Deserialize)]
struct SXceiverDto {
    pub rx_ant: Option<String>,
    pub tx_ant: Option<String>,
    pub rx_gain_lna: Option<f64>,
    pub rx_gain_pga: Option<f64>,
    pub tx_gain_dac: Option<f64>,
    pub tx_gain_mixer: Option<f64>,
}

#[derive(Default, Deserialize)]
struct NetInfoDto {
    pub mcc: Option<u16>,
    pub mnc: Option<u16>,

    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[derive(Default, Deserialize)]
struct CellInfoDto {

    pub main_carrier: u16,
    pub freq_band: u8,
    pub freq_offset: i16,
    pub duplex_spacing: u8,
    pub reverse_operation: bool,
    pub custom_duplex_spacing: Option<u32>,

    pub location_area: u16,
    
    pub neighbor_cell_broadcast: Option<u8>,
    pub cell_load_ca: Option<u8>,
    pub late_entry_supported: Option<bool>,


    

    pub subscriber_class: Option<u16>,

    pub registration: Option<bool>,
    pub deregistration: Option<bool>,
    pub priority_cell: Option<bool>,
    pub no_minimum_mode: Option<bool>,
    pub migration: Option<bool>,
    pub system_wide_services: Option<bool>,
    pub voice_service: Option<bool>,
    pub circuit_mode_data_service: Option<bool>,
    pub sndcp_service: Option<bool>,
    pub aie_service: Option<bool>,
    pub advanced_link: Option<bool>,

    pub system_code: Option<u8>,
    pub colour_code: Option<u8>,
    pub sharing_mode: Option<u8>,
    pub ts_reserved_frames: Option<u8>,
    pub u_plane_dtx: Option<bool>,
    pub frame_18_ext: Option<bool>,

    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[derive(Default, Deserialize)]
struct StackStatePatch {
    pub cell_load_ca: Option<u8>,

    #[serde(flatten)]
    extra: HashMap<String, Value>,
}
