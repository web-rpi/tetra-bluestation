use crate::config::stack_config::SharedConfig;
use crate::common::messagerouter::MessageQueue;
use crate::entities::mm::enums::mm_location_update_type::MmLocationUpdateType;
use crate::entities::mm::fields::group_identity_location_accept::GroupIdentityLocationAccept;
use crate::entities::mm::fields::group_identity_uplink::GroupIdentityUplink;
use crate::saps::lmm::LmmMleUnitdataReq;
use crate::saps::sapmsg::{SapMsg, SapMsgInner};
use crate::common::address::{SsiType, TetraAddress};
use crate::common::bitbuffer::BitBuffer;
use crate::entities::mm::components::client_state::MmClientMgr;
use crate::entities::mm::enums::mm_pdu_type_ul::MmPduTypeUl;
use crate::entities::mm::fields::group_identity_attachment::GroupIdentityAttachment;
use crate::entities::mm::fields::group_identity_downlink::GroupIdentityDownlink;
use crate::entities::mm::pdus::d_attach_detach_group_identity_acknowledgement::DAttachDetachGroupIdentityAcknowledgement;
use crate::entities::mm::pdus::d_location_update_accept::DLocationUpdateAccept;
use crate::entities::mm::pdus::u_attach_detach_group_identity::UAttachDetachGroupIdentity;
use crate::entities::mm::pdus::u_itsi_detach::UItsiDetach;
use crate::entities::mm::pdus::u_location_update_demand::ULocationUpdateDemand;
use crate::entities::TetraEntityTrait;
use crate::common::tetra_common::Sap;
use crate::common::tetra_entities::TetraEntity;
use crate::unimplemented_log;

pub struct MmBs {
    config: SharedConfig,
    pub client_mgr: MmClientMgr,
}

impl MmBs {
    pub fn new(config: SharedConfig) -> Self {
        Self { config, client_mgr: MmClientMgr::new() }
    }

    fn rx_u_itsi_detach(&mut self, _queue: &mut MessageQueue, mut message: SapMsg) {
        tracing::trace!("rx_u_itsi_detach: {:?}", message);
        let SapMsgInner::LmmMleUnitdataInd(prim) = &mut message.msg else {panic!()};
        
        let _pdu = match UItsiDetach::from_bitbuf(&mut prim.sdu) {
            Ok(pdu) => {
                tracing::debug!("<- {:?}", pdu);
                pdu
            }
            Err(e) => {
                tracing::warn!("Failed parsing UItsiDetach: {:?} {}", e, prim.sdu.dump_bin());
                return;
            }
        };

        let ssi = prim.received_address.ssi;
        let detached_client = self.client_mgr.remove_client(ssi);
        if detached_client.is_none() {
            tracing::warn!("Received UItsiDetach for unknown client with SSI: {}", ssi);
            // return;
        };
    }

    fn rx_u_location_update_demand(&mut self, queue: &mut MessageQueue, mut message: SapMsg) {
        tracing::trace!("rx_location_update_demand: {:?}", message);
        let SapMsgInner::LmmMleUnitdataInd(prim) = &mut message.msg else {panic!()};

        let pdu = match ULocationUpdateDemand::from_bitbuf(&mut prim.sdu) {
            Ok(pdu) => {
                tracing::debug!("<- {:?}", pdu);
                pdu
            }
            Err(e) => {
                tracing::warn!("Failed parsing ULocationUpdateDemand: {:?} {}", e, prim.sdu.dump_bin());
                return;
            }
        };

        // Check if we can satisfy this request, print unsupported stuff
        if !Self::feature_check_u_location_update_demand(&pdu) {
            tracing::error!("Unsupported features in ULocationUpdateDemand");
            return;
        }

        // Try to register the client
        let issi = prim.received_address.ssi;
        match self.client_mgr.register_client(issi, true) {
            Ok(_) => {},
            Err(e) => {
                tracing::warn!("Failed registering roaming MS {}: {:?}", issi, e);
                unimplemented_log!("Handle failed registration of roaming MS");
                return;
            }
        }

        // Process optional GroupIdentityLocationDemand field
        let gila = if let Some(gild) = pdu.group_identity_location_demand {
            
            // Try to attach to requested groups, then build GroupIdentityLocationAccept element
            let accepted_groups = if let Some(giu) = &gild.group_identity_uplink {
                Some(self.try_attach_detach_groups(issi, &giu))
            } else {
                None
            };
            let gila = GroupIdentityLocationAccept {
                group_identity_accept_reject: 0, // Accept
                group_identity_downlink: accepted_groups,
            };

            Some(gila)
        } else {
            // No GroupIdentityLocationAccept element present
            None
        };

        // Build D-LOCATION UPDATE ACCEPT pdu
        let pdu_response = DLocationUpdateAccept {
            location_update_accept_type: pdu.location_update_type, // Practically identical besides minor migration-related difference
            ssi: Some(issi as u64),
            address_extension: None,
            subscriber_class: None,
            energy_saving_information: None,
            scch_information_and_distribution_on_18th_frame: None,
            new_registered_area: None,
            security_downlink: None,
            group_identity_location_accept: gila,
            default_group_attachment_lifetime: None,
            authentication_downlink: None,
            group_identity_security_related_information: None,
            cell_type_control: None,
            proprietary: None,
        };

        // Convert pdu to bits
        let pdu_len = 4+3+24+1+1+1; // Minimal lenght; may expand beyond this. 
        let mut sdu = BitBuffer::new_autoexpand(pdu_len);
        pdu_response.to_bitbuf(&mut sdu).unwrap(); // we want to know when this happens
        sdu.seek(0);
        tracing::debug!("-> {} sdu {}", pdu_response, sdu.dump_bin());

        // Build and submit response prim
        let addr = TetraAddress { encrypted: false, ssi_type: SsiType::Ssi, ssi: issi };
        let msg = SapMsg {
            sap: Sap::LmmSap,
            src: TetraEntity::Mm,
            dest: TetraEntity::Mle,
            dltime: message.dltime,
            msg: SapMsgInner::LmmMleUnitdataReq(LmmMleUnitdataReq{
                sdu,
                handle: 0,
                address: addr,
                layer2service: 0,
                stealing_permission: false,
                stealing_repeats_flag: false, 
                encryption_flag: false,
                is_null_pdu: false,
            })
        };
        queue.push_back(msg);        
    }


    fn rx_u_attach_detach_group_identity(&mut self, queue: &mut MessageQueue, mut message: SapMsg) {
        tracing::trace!("rx_u_attach_detach_group_identity: {:?}", message);
        let SapMsgInner::LmmMleUnitdataInd(prim) = &mut message.msg else {panic!()};
        
        let issi = prim.received_address.ssi;
        let pdu = match UAttachDetachGroupIdentity::from_bitbuf(&mut prim.sdu) {
            Ok(pdu) => {
                tracing::debug!("<- {:?}", pdu);
                pdu
            }
            Err(e) => {
                tracing::warn!("Failed parsing UAttachDetachGroupIdentity: {:?} {}", e, prim.sdu.dump_bin());
                return;
            }
        };

        // Check if we can satisfy this request, print unsupported stuff
        if !Self::feature_check_u_attach_detach_group_identity(&pdu) {
            tracing::error!("Unsupported features in UAttachDetachGroupIdentity");
            return;
        }

        // If group_identity_attach_detach_mode == 1, we first detach all groups
        if pdu.group_identity_attach_detach_mode == true {
            match self.client_mgr.client_detach_all_groups(issi) {
                Ok(_) => {},
                Err(e) => {
                    tracing::warn!("Failed detaching all groups for MS {}: {:?}", issi, e);
                    return;
                }
            }
        }

        // Try to attach to requested groups, and retrieve list of accepted GroupIdentityDownlink elements
        // We can unwrap since we did compat check earlier
        let accepted_gid= self.try_attach_detach_groups(issi, &pdu.group_identity_uplink.unwrap());

        // Build reply PDU
        let pdu_response = DAttachDetachGroupIdentityAcknowledgement {
            group_identity_accept_reject: 0, // Accept
            reserved: false, // TODO FIXME Guessed proper value of reserved field
            proprietary: None,
            group_identity_downlink: Some(accepted_gid),
            group_identity_security_related_information: None,
        };

        // Write to PDU
        let mut sdu = BitBuffer::new_autoexpand(32);
        pdu_response.to_bitbuf(&mut sdu).unwrap(); // We want to know when this happens
        sdu.seek(0);
        tracing::debug!("-> {:?} sdu {}", pdu_response, sdu.dump_bin());

        let addr = TetraAddress { 
            encrypted: false, 
            ssi_type: SsiType::Ssi, 
            ssi: issi 
        };
        let msg = SapMsg {
            sap: Sap::LmmSap,
            src: TetraEntity::Mm,
            dest: TetraEntity::Mle,
            dltime: message.dltime,
            msg: SapMsgInner::LmmMleUnitdataReq(LmmMleUnitdataReq{
                sdu,
                handle: 0,
                address: addr,
                layer2service: 0,
                stealing_permission: false,
                stealing_repeats_flag: false, 
                encryption_flag: false,
                is_null_pdu: false,
            })
        };
        queue.push_back(msg);
    }

    fn rx_lmm_mle_unitdata_ind(&mut self, queue: &mut MessageQueue, mut message: SapMsg) {

        // unimplemented_log!("rx_lmm_mle_unitdata_ind for MM component");
        let SapMsgInner::LmmMleUnitdataInd(prim) = &mut message.msg else {panic!()};

        let Some(bits) = prim.sdu.peek_bits(4) else {
            tracing::warn!("insufficient bits: {}", prim.sdu.dump_bin());
            return;
        };

        let Ok(pdu_type) = MmPduTypeUl::try_from(bits) else {
            tracing::warn!("invalid pdu type: {} in {}", bits, prim.sdu.dump_bin());
            return;
        };

        match pdu_type {
            MmPduTypeUl::UAuthentication => 
                unimplemented_log!("UAuthentication"),
            MmPduTypeUl::UItsiDetach => 
                self.rx_u_itsi_detach(queue, message),
            MmPduTypeUl::ULocationUpdateDemand => 
                self.rx_u_location_update_demand(queue, message),
            MmPduTypeUl::UMmStatus =>   
                unimplemented_log!("UMmStatus"),
            MmPduTypeUl::UCkChangeResult => 
                unimplemented_log!("UCkChangeResult"),
            MmPduTypeUl::UOtar =>   
                unimplemented_log!("UOtar"),
            MmPduTypeUl::UInformationProvide => 
                unimplemented_log!("UInformationProvide"),
            MmPduTypeUl::UAttachDetachGroupIdentity => 
                self.rx_u_attach_detach_group_identity(queue, message),
            MmPduTypeUl::UAttachDetachGroupIdentityAcknowledgement => 
                unimplemented_log!("UAttachDetachGroupIdentityAcknowledgement"),
            MmPduTypeUl::UTeiProvide => 
                unimplemented_log!("UTeiProvide"),
            MmPduTypeUl::UDisableStatus => 
                unimplemented_log!("UDisableStatus"),
            MmPduTypeUl::MmPduFunctionNotSupported => 
                unimplemented_log!("MmPduFunctionNotSupported"),
        };
    }

    fn try_attach_detach_groups(&mut self, issi: u32, giu_vec: &Vec<GroupIdentityUplink>) -> Vec<GroupIdentityDownlink> {
        let mut accepted_groups = Vec::new();
        for giu in giu_vec.iter() {
            if giu.gssi.is_none() || giu.vgssi.is_some() || giu.address_extension.is_some() {
                unimplemented_log!("Only support GroupIdentityUplink with address_type 0");
                continue;
            }   

            let gssi = giu.gssi.unwrap(); // can't fail
            match self.client_mgr.client_group_attach(issi, gssi, true) {
                Ok(_) => {
                    // We have added the client to this group. Add an entry to the downlink response
                    let gid = GroupIdentityDownlink {
                        group_identity_attachment: Some(GroupIdentityAttachment {
                            group_identity_attachment_lifetime: 3, // re-attach after location update
                            class_of_usage: giu.class_of_usage.unwrap_or(0),
                        }),
                        group_identity_detachment_uplink: None,
                        gssi: Some(giu.gssi.unwrap()),
                        address_extension: None,
                        vgssi: None
                    };
                    accepted_groups.push(gid);
                },
                Err(e) => {
                    tracing::warn!("Failed attaching MS {} to group {}: {:?}", issi, gssi, e);
                }
            }
        }
        accepted_groups
    }

    fn feature_check_u_location_update_demand(pdu: &ULocationUpdateDemand) -> bool {
        let mut supported = true;
        if pdu.location_update_type != MmLocationUpdateType::RoamingLocationUpdating && pdu.location_update_type != MmLocationUpdateType::ItsiAttach {
            unimplemented_log!("Unsupported {}", pdu.location_update_type);
            supported = false;
        }
        if pdu.request_to_append_la == true {
            unimplemented_log!("Unsupported request_to_append_la == true");
            supported = false;
        }
        if pdu.cipher_control == true {
            unimplemented_log!("Unsupported cipher_control == true");
            supported = false;
        }
        if pdu.ciphering_parameters.is_some() {
            unimplemented_log!("Unsupported ciphering_parameters present");
            supported = false;
        }
        // pub class_of_ms: Option<u64>, currently not parsed nor interpreted
        if pdu.energy_saving_mode.is_some() {
            unimplemented_log!("Unsupported energy_saving_mode present");
        }
        if pdu.la_information.is_some() {
            unimplemented_log!("Unsupported la_information present");
        }
        if pdu.ssi.is_some() {
            unimplemented_log!("Unsupported ssi present");
        }
        if pdu.address_extension.is_some() {
            unimplemented_log!("Unsupported address_extension present");
        }
        // pub group_identity_location_demand: Option<GroupIdentityLocationDemand>, kind of supported
        if pdu.group_report_response.is_some() {
            unimplemented_log!("Unsupported group_report_response present");
        }
        if pdu.authentication_uplink.is_some() {
            unimplemented_log!("Unsupported authentication_uplink present");
        }
        if pdu.extended_capabilities.is_some() {
            unimplemented_log!("Unsupported extended_capabilities present");
        }
        if pdu.proprietary.is_some() {
            unimplemented_log!("Unsupported proprietary present");
        }

        supported
    }


    fn feature_check_u_attach_detach_group_identity(pdu: &UAttachDetachGroupIdentity) -> bool {
        let mut supported = true;
        if pdu.group_identity_report == true {
            unimplemented_log!("Unsupported group_identity_report == true");
        }
        if pdu.group_identity_uplink.is_none() {
            unimplemented_log!("Missing group_identity_uplink");
            supported = false;
        }
        if pdu.group_report_response.is_some() {
            unimplemented_log!("Unsupported group_report_response present");
        }
        if pdu.proprietary.is_some() {
            unimplemented_log!("Unsupported proprietary present");
        }

        supported
    }
}



impl TetraEntityTrait for MmBs {

    fn entity(&self) -> TetraEntity {
        TetraEntity::Mm
    }

    fn set_config(&mut self, config: SharedConfig) {
        self.config = config;
    }

    fn rx_prim(&mut self, queue: &mut MessageQueue, message: SapMsg) {
        
        tracing::debug!("rx_prim: {:?}", message);
        
        // There is only one SAP for MM
        assert!(message.sap == Sap::LmmSap);
        
        match message.msg {
            SapMsgInner::LmmMleUnitdataInd(_) => {
                self.rx_lmm_mle_unitdata_ind(queue, message);
            }
            _ => { panic!(); }
        }
    }
}
