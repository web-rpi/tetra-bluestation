use std::collections::HashMap;

use tetra_core::{BitBuffer, Direction, Sap, SsiType, TdmaTime, TetraAddress, tetra_entities::TetraEntity, unimplemented_log};
use tetra_pdus::cmce::{enums::{call_timeout::CallTimeout, call_timeout_setup_phase::CallTimeoutSetupPhase, cmce_pdu_type_ul::CmcePduTypeUl, transmission_grant::TransmissionGrant}, fields::basic_service_information::BasicServiceInformation, pdus::{d_call_proceeding::DCallProceeding, d_connect::DConnect, d_release::DRelease, d_setup::DSetup, u_setup::USetup}, structs::cmce_circuit::CmceCircuit};
use tetra_saps::{SapMsg, SapMsgInner, control::{call_control::{CallControl, Circuit}, enums::communication_type::CommunicationType}, lcmc::{LcmcMleUnitdataReq, enums::{alloc_type::ChanAllocType, ul_dl_assignment::UlDlAssignment}, fields::chan_alloc_req::CmceChanAllocReq}};

use crate::{MessageQueue, cmce::components::circuit_mgr::{CircuitMgr, CircuitMgrCmd}};


/// Clause 11 Call Control CMCE sub-entity
pub struct CcBsSubentity{
    dltime: TdmaTime,
    cached_setups: HashMap<u16, DSetup>,
    circuits: CircuitMgr,
}

impl CcBsSubentity {
    
    pub fn new() -> Self {
        CcBsSubentity {
            dltime: TdmaTime::default(),
            cached_setups: HashMap::new(),
            circuits: CircuitMgr::new(),
        }
    }

    pub fn run_call_test(&mut self, queue: &mut MessageQueue, dltime: TdmaTime) {

        tracing::error!("-------- Running call test -------");
        
        // Create a new circuit
        let circuit = match self.circuits.allocate_circuit(
            Direction::Dl, 
            CommunicationType::P2Mp) 
        {
            Ok(circuit) => circuit,
            Err(e) => {
                tracing::error!("Failed to allocate circuit for call test: {:?}", e);
                return;
            }
        };  

        // Signal UMAC to setup the circuit
        Self::signal_umac_circuit_open(queue, &circuit, dltime);

        // Build D-SETUP PDU and send down the stack
        let pdu_d_setup = Self::build_d_setup_pdu_from_circuit(&circuit);
        self.cached_setups.insert(circuit.call_id, pdu_d_setup);
        let pdu_ref = self.cached_setups.get(&circuit.call_id).unwrap();

        let (pdu, chan_alloc) = Self::build_d_setup_prim_from_pdu(pdu_ref, circuit.usage);
        let prim = Self::build_sapmsg(pdu, Some(chan_alloc), dltime);
        queue.push_back(prim);
    }

    fn build_d_setup_pdu_from_circuit(circuit: &CmceCircuit) -> DSetup {
        DSetup { 
            call_identifier: circuit.call_id,
            call_time_out: CallTimeout::Infinite, 
            hook_method_selection: false, 
            simplex_duplex_selection: circuit.simplex_duplex, 
            basic_service_information: BasicServiceInformation { 
                circuit_mode_type: circuit.circuit_mode,
                encryption_flag: circuit.etee_encrypted, 
                communication_type: circuit.comm_type, 
                slots_per_frame: None, 
                speech_service: Some(0) 
            }, 
            transmission_grant: TransmissionGrant::NotGranted, 
            transmission_request_permission: false, 
            call_priority: 0, 
            notification_indicator: None, 
            temporary_address: None, 
            calling_party_address_ssi: Some(2041234), 
            calling_party_extension: None, 
            external_subscriber_number: None, 
            facility: None, 
            dm_ms_address: None, 
            proprietary: None 
        }
    }

    fn build_d_setup_prim_from_pdu(pdu: &DSetup, usage: u8) -> (BitBuffer, CmceChanAllocReq) {

        tracing::info!("-> {:?}", pdu);

        let mut sdu = BitBuffer::new_autoexpand(80);
        pdu.to_bitbuf(&mut sdu).expect("Failed to serialize DSetup");
        sdu.seek(0);
        
        // Construct ChanAlloc descriptor
        let chan_alloc = CmceChanAllocReq {
            usage: Some(usage),
            alloc_type: ChanAllocType::Replace,
            carrier: None,
            timeslots: [false, true, false, false],
            ul_dl_assigned: UlDlAssignment::Dl,
        };
        (sdu, chan_alloc)
    }

    fn build_sapmsg(sdu: BitBuffer, chan_alloc: Option<CmceChanAllocReq>, dltime: TdmaTime) -> SapMsg {

        // Construct prim
        SapMsg {
            sap: Sap::LcmcSap,
            src: TetraEntity::Cmce,
            dest: TetraEntity::Mle,
            dltime: dltime,
            msg: SapMsgInner::LcmcMleUnitdataReq(LcmcMleUnitdataReq{
                sdu: sdu,
                handle: 0,
                endpoint_id: 0,
                link_id: 0,
                layer2service: 0,
                pdu_prio: 0,
                layer2_qos: 0,
                stealing_permission: false,
                stealing_repeats_flag: false,
                chan_alloc,
                main_address: TetraAddress::new(26, SsiType::Gssi),
            })
        }
    }

    fn build_d_release_from_d_setup(d_setup_pdu: &DSetup) -> BitBuffer {

        let pdu = DRelease {
            call_identifier: d_setup_pdu.call_identifier,
            disconnect_cause: 13, // todo fixme
            notification_indicator: None,
            facility: None,
            proprietary: None,
        };
        tracing::info!("-> {:?}", pdu);
                        
        let mut sdu = BitBuffer::new_autoexpand(32);
        pdu.to_bitbuf(&mut sdu).expect("Failed to serialize DRelease");
        sdu.seek(0);
        sdu
    }

    fn send_d_call_proceeding(&mut self, queue: &mut MessageQueue, message: &SapMsg, pdu_request: &USetup, call_id: u16) {
        tracing::trace!("send_d_call_proceeding");

        let SapMsgInner::LcmcMleUnitdataInd(prim) = &message.msg else {panic!()};
        
        let pdu_response = DCallProceeding {
            call_identifier: call_id,
            call_time_out_set_up_phase: CallTimeoutSetupPhase::T10s,
            hook_method_selection: pdu_request.hook_method_selection,
            simplex_duplex_selection: pdu_request.simplex_duplex_selection,
            basic_service_information: None, // Only needed if different from requested
            call_status: None,
            notification_indicator: None,
            facility: None,
            proprietary: None,
        };
        
        let mut sdu = BitBuffer::new_autoexpand(25);
        pdu_response.to_bitbuf(&mut sdu).expect("Failed to serialize DCallProceeding");
        sdu.seek(0);
        tracing::debug!("send_d_call_proceeding: -> {:?} sdu {}", pdu_response, sdu.dump_bin());

        let msg = SapMsg {
            sap: Sap::LcmcSap,
            src: TetraEntity::Cmce,
            dest: TetraEntity::Mle,
            dltime: message.dltime,
            msg: SapMsgInner::LcmcMleUnitdataReq(LcmcMleUnitdataReq{
                sdu: sdu,
                handle: prim.handle,
                endpoint_id: prim.endpoint_id,
                link_id: prim.link_id,
                layer2service: 0,
                pdu_prio: 0,
                layer2_qos: 0,
                stealing_permission: false,
                stealing_repeats_flag: false,

                chan_alloc: None,
                main_address: prim.received_tetra_address,
                // redundant_transmission: 1,
            })
        };
        queue.push_back(msg);
    }


    fn send_d_connect(&mut self, queue: &mut MessageQueue, message: &SapMsg, pdu_request: &USetup, call_id: u16) {
        tracing::trace!("send_d_connect");

        let SapMsgInner::LcmcMleUnitdataInd(prim) = &message.msg else {panic!()};

        let pdu_response = DConnect {
            call_identifier: call_id,
            call_time_out: CallTimeout::T30m,
            hook_method_selection: pdu_request.hook_method_selection,
            simplex_duplex_selection: pdu_request.simplex_duplex_selection,
            transmission_grant: TransmissionGrant::Granted,
            transmission_request_permission: false, // CHECKME an MS may not ask for transmit permission
            call_ownership: false, // Group call meaning: false = not a call owner
            call_priority: None,
            basic_service_information: None,
            temporary_address: None,
            notification_indicator: None,
            facility: None,
            proprietary: None,
        };
        
        let mut sdu = BitBuffer::new_autoexpand(30);
        pdu_response.to_bitbuf(&mut sdu).expect("Failed to serialize DConnect");
        sdu.seek(0);
        tracing::debug!("send_d_connect: -> {:?} sdu {}", pdu_response, sdu.dump_bin());

        let msg = SapMsg {
            sap: Sap::LcmcSap,
            src: TetraEntity::Cmce,
            dest: TetraEntity::Mle,
            dltime: message.dltime,
            msg: SapMsgInner::LcmcMleUnitdataReq(LcmcMleUnitdataReq{
                sdu: sdu,
                handle: prim.handle,
                endpoint_id: prim.endpoint_id,
                link_id: prim.link_id,
                layer2service: 0,
                pdu_prio: 0,
                layer2_qos: 0,
                stealing_permission: false,
                stealing_repeats_flag: false,
                
                chan_alloc: None,
                main_address: prim.received_tetra_address,
                // redundant_transmission: 1,
            })
        };
        queue.push_back(msg);
    }


    // fn send_d_setup(&mut self, queue: &mut MessageQueue, message: &SapMsg, pdu_request: &USetup, call_id: u16, calling_party: TetraAddress) {
    //     tracing::trace!("send_d_setup");

    //     let SapMsgInner::LcmcMleUnitdataInd(prim) = &message.msg else {panic!()};

    //     let transmission_grant = match pdu_request.request_to_transmit_send_data {
    //         true => TransmissionGrant::Granted,
    //         false => TransmissionGrant::NotGranted,
    //     };        

    //     let pdu_response = DSetup {
    //         call_identifier: call_id,
    //         call_time_out: CallTimeout::T5m,
    //         hook_method_selection: pdu_request.hook_method_selection,
    //         simplex_duplex_selection: pdu_request.simplex_duplex_selection,
    //         basic_service_information: pdu_request.basic_service_information.clone(),
    //         transmission_grant: transmission_grant,
    //         transmission_request_permission: false,
    //         call_priority: 0,
    //         temporary_address: None,
    //         calling_party_address_ssi: Some(calling_party.ssi),
    //         calling_party_extension: None,
    //         external_subscriber_number: None,
    //         dm_ms_address: None,
    //         notification_indicator: None,
    //         facility: None,
    //         proprietary: None,
    //     };
        
    //     let mut sdu = BitBuffer::new_autoexpand(71);
    //     pdu_response.to_bitbuf(&mut sdu).expect("Failed to serialize DSetup");
    //     sdu.seek(0);
    //     tracing::debug!("send_d_setup: -> {:?} sdu {}", pdu_response, sdu.dump_bin());

    //     let chan_alloc = Some(CmceChanAllocReq {
    //         usage: self.circuit_alloc_usage(),
    //         alloc_type: ChanAllocType::Replace,
    //         carrier: None,
    //         timeslots: [false, true, false, false],
    //         ul_dl_assigned: UlDlAssignment::Both,
    //     });
        
    //     let msg = SapMsg {
    //         sap: Sap::LcmcSap,
    //         src: TetraEntity::Cmce,
    //         dest: TetraEntity::Mle,
    //         dltime: message.dltime,
    //         msg: SapMsgInner::LcmcMleUnitdataReq(LcmcMleUnitdataReq{
    //             sdu: sdu,
    //             handle: prim.handle,
    //             endpoint_id: prim.endpoint_id,
    //             link_id: prim.link_id,
    //             layer2service: 0,
    //             pdu_prio: 0,
    //             layer2_qos: 0,
    //             stealing_permission: false,
    //             stealing_repeats_flag: false,
    //             chan_alloc,
    //             main_address: prim.received_tetra_address,
    //             // redundant_transmission: 4,
    //         })
    //     };
    //     queue.push_back(msg);
    // }

    fn signal_umac_circuit_open(queue: &mut MessageQueue, call: &CmceCircuit, dltime: TdmaTime) {
        let circuit = Circuit {
            direction: call.direction,
            ts: call.ts,            
            usage: call.usage,
            circuit_mode: call.circuit_mode,
            speech_service: call.speech_service,
            etee_encrypted: call.etee_encrypted,
        };        
        let cmd = SapMsg {
            sap: Sap::Control,
            src: TetraEntity::Cmce,
            dest: TetraEntity::Umac,
            dltime: dltime,
            msg: SapMsgInner::CmceCallControl(
                CallControl::Open(circuit)
            ),
        };
        queue.push_back(cmd);
    }

    fn signal_umac_circuit_close(queue: &mut MessageQueue, circuit: CmceCircuit, dltime: TdmaTime) {
        let cmd = SapMsg {
            sap: Sap::Control,
            src: TetraEntity::Cmce,
            dest: TetraEntity::Umac,
            dltime: dltime,
            msg: SapMsgInner::CmceCallControl(
                CallControl::Close(circuit.direction, circuit.ts)
            ),
        };
        queue.push_back(cmd);
    }

    fn rx_u_setup(&mut self, _queue: &mut MessageQueue, mut message: SapMsg) {
        tracing::trace!("rx_u_setup: {:?}", message);
        let SapMsgInner::LcmcMleUnitdataInd(prim) = &mut message.msg else {panic!()};
        // let calling_party = prim.received_tetra_address.clone();
        
        let pdu = match USetup::from_bitbuf(&mut prim.sdu) {
            Ok(pdu) => {
                tracing::debug!("<- {:?}", pdu);
                pdu
            }
            Err(e) => {
                tracing::warn!("Failed parsing UItsiDetach: {:?} {}", e, prim.sdu.dump_bin());
                return;
            }
        };

        // Check if we can satisfy this request, print unsupported stuff
        if !Self::feature_check_u_setup(&pdu) {
            tracing::error!("Unsupported critical features in USetup");
            return;
        }

        // let tx_grant = pdu.request_to_transmit_send_data;

        // Let's reserve an identifier, create the call FSM
        // let call_id = self.get_next_call_identifier();
        // let call_ts = self.circuit_get_free_ts().expect("No free timeslot for new call"); // TODO FIXME implement proper handling here
        // let call = CmceCircuit::from_u_setup(Direction::Both, call_id, call_ts, &pdu);
        
        // tracing::info!("Creating call {:?}", call);

        unimplemented_log!("rx_u_setup");


        // if tx_grant {
            // self.signal_umac_circuit_setup(queue, &call, message.dltime);
        // }
        
        // self.send_d_call_proceeding(queue, &message, &pdu, call_id);
        // self.send_d_connect(queue, &message, &pdu, call_id);
        // self.send_d_setup(queue, &message, &pdu, call_id, calling_party);

        // Also send control message to ultimately instruct MAC to go into traffic mode. 
        // let ctl_msg = Self::notify_lmac_new_call(message.dltime, call, tx_grant);
        // queue.push_back(ctl_msg);
    }

    pub fn route_xx_deliver(&mut self, _queue: &mut MessageQueue, mut message: SapMsg) {
        
        tracing::trace!("route_xx_deliver");
        
        let SapMsgInner::LcmcMleUnitdataInd(prim) = &mut message.msg else { panic!(); };
        let Some(bits) = prim.sdu.peek_bits(5) else {
            tracing::warn!("insufficient bits: {}", prim.sdu.dump_bin());
            return;
        };
        let Ok(pdu_type) = CmcePduTypeUl::try_from(bits) else {
            tracing::warn!("invalid pdu type: {} in {}", bits, prim.sdu.dump_bin());
            return;
        };

        // TODO FIXME: Besides these PDUs, we can also receive several signals (BUSY ind, CLOSE ind, etc)
        match pdu_type {
            CmcePduTypeUl::USetup => 
                self.rx_u_setup(_queue, message),
            CmcePduTypeUl::UAlert |
            CmcePduTypeUl::UConnect |
            CmcePduTypeUl::UDisconnect |
            CmcePduTypeUl::UInfo |
            CmcePduTypeUl::URelease |
            CmcePduTypeUl::UStatus |
            CmcePduTypeUl::UTxCeased |
            CmcePduTypeUl::UTxDemand |
            CmcePduTypeUl::UCallRestore => {
                unimplemented_log!("{}", pdu_type);
            }
            _ => {
                panic!();
            }
        }
    }

    pub fn tick_start(&mut self, queue: &mut MessageQueue, dltime: TdmaTime) {
        self.dltime = dltime;
        if let Some(tasks) = self.circuits.tick_start(dltime) {
            for task in tasks {
                match task {
                    CircuitMgrCmd::SendDSetup(call_id, usage) => {
                        // Get our cached D-SETUP, build a prim and send it down the stack
                        let Some(pdu) = self.cached_setups.get(&call_id) else {
                            tracing::error!("No cached D-SETUP for call id {}", call_id);
                            return; 
                        };
                        tracing::info!("-> {:?}", pdu);
                        let (pdu, chan_alloc) = Self::build_d_setup_prim_from_pdu(pdu, usage);
                        let prim = Self::build_sapmsg(pdu, Some(chan_alloc), self.dltime);
                        queue.push_back(prim);
                    },

                    CircuitMgrCmd::SendClose(call_id, circuit) => {
                        tracing::warn!("need to send CLOSE for call id {}", call_id);
                        // Get our cached D-SETUP, build a prim and send it down the stack
                        let Some(pdu) = self.cached_setups.get(&call_id) else {
                            tracing::error!("No cached D-SETUP for call id {}", call_id);
                            return; 
                        };
                        
                        let sdu = Self::build_d_release_from_d_setup(pdu);
                        let prim = Self::build_sapmsg(sdu, None, self.dltime);
                        queue.push_back(prim);

                        // Signal UMAC to release the circuit
                        Self::signal_umac_circuit_close(queue, circuit, self.dltime);
                    }
                }
            }
        }
    }

    fn feature_check_u_setup(pdu: &USetup) -> bool {
        let mut supported = true;

        if !(pdu.area_selection == 0 || pdu.area_selection == 1) {
            unimplemented_log!("Area selection not supported: {}", pdu.area_selection);
            supported = false;
        };
        if pdu.hook_method_selection == true {
            unimplemented_log!("Hook method selection not supported: {}", pdu.hook_method_selection);
            supported = false;
        };
        if pdu.simplex_duplex_selection != false {
            unimplemented_log!("Only simplex calls supported: {}", pdu.simplex_duplex_selection);
            supported = false;
        };
        // if pdu.basic_service_information != 0xFC {
        //     // TODO FIXME implement parsing
        //     tracing::error!("Basic service information not supported: {}", pdu.basic_service_information);
        //     return;
        // };
        if pdu.request_to_transmit_send_data != true {
            unimplemented_log!("Expect request_to_transmit, need to revisit FSM for this value");      
            // supported = false;
        };
        if pdu.clir_control != 0 {
            unimplemented_log!("clir_control not supported: {}", pdu.clir_control);
        };
        if pdu.called_party_ssi.is_none() || pdu.called_party_short_number_address.is_some() || pdu.called_party_extension.is_some() {
            unimplemented_log!("we only support ssi-based calling");
        };
        // Then, we warn about some other unhandled/unsupported fields
        if let Some(v) = &pdu.external_subscriber_number {
            unimplemented_log!("external_subscriber_number not supported: {:?}", v);
        };
        if let Some(v) = &pdu.facility {
            unimplemented_log!("facility not supported: {:?}", v);
        };
        if let Some(v) = &pdu.dm_ms_address {
            unimplemented_log!("dm_ms_address not supported: {:?}", v);
        };
        if let Some(v) = &pdu.proprietary {
            unimplemented_log!("proprietary not supported: {:?}", v);
        };

        supported
    }
}