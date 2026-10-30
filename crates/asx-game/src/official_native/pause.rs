//! Immutable native pause contacts supplied only by an exact selected adapter.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PauseLayout {
    addresses: [usize; 13],
}

impl PauseLayout {
    pub(crate) const fn new(addresses: [usize; 13]) -> Self {
        Self { addresses }
    }

    pub fn rva(&self, key: usize) -> usize {
        const KEYS: [usize; 13] = [
            0x1bf9c, 0x1bfcd, 0x346580, 0x346b4c, 0x346d29, 0x346cc9,
            0x1bbce, 0x3227b0, 0x31ddd0, 0x1bfc1, 0x1b6d4, 0x346ac3, 0x347403,
        ];
        self.addresses[KEYS.iter().position(|value| *value == key).expect("Unreviewed pause address")]
    }
}

#[cfg(test)]
mod tests {
    use crate::official::SelectedAdapter;

    #[test]
    fn selected_layout_preserves_all_reviewed_contacts() {
        let adapter = SelectedAdapter::select(crate::current::SHA256).unwrap();
        let layout = adapter.pause_layout();
        let keys = [
            0x1bf9c, 0x1bfcd, 0x346580, 0x346b4c, 0x346d29, 0x346cc9,
            0x1bbce, 0x3227b0, 0x31ddd0, 0x1bfc1, 0x1b6d4, 0x346ac3, 0x347403,
        ];
        assert_eq!(keys.map(|key| layout.rva(key)), adapter.pause_addresses());
    }
}

// Native 0.130 stop builder 0x5b4cab0: packed GUID, xyz, spline timestamp,
// MonsterMoveStop (1). This stops only the recipient's rendering spline.
fn stop_body(guid:u64,position:[f32;3],clock:f32)->Result<Vec<u8>,String>{
    if guid==0||position.iter().any(|v|!v.is_finite())||!clock.is_finite()||clock<0.||clock>1e7{return Err("Invalid pause motion identity/pose".into())}
    let bytes=guid.to_le_bytes();let mask=bytes.iter().enumerate().fold(0u8,|m,(i,b)|m|if *b!=0{1<<i}else{0});
    let mut body=vec![mask];for b in bytes{if b!=0{body.push(b)}}
    for v in position{body.extend_from_slice(&v.to_le_bytes())}
    body.extend_from_slice(&((clock*1000.)as u32).to_le_bytes());body.push(1);Ok(body)
}
impl super::Adapter {
    /// One pause-entry update, through the existing native packet sender.
    /// Do not rewrite actor state, local-player movement or network clocks.
    pub unsafe fn pause_remote_motion(self,m:&dyn crate::memory::Mem,base:u64,game:u64)->Result<usize,String>{
        let Some(party)=self.world_party(m,game)?else{return Ok(0)};
        if self.run_source(m,game)?.is_none_or(|(slot,_)|slot!=0){return Err("Pause motion requires restored host authority".into())}
        let (rva,bytes)=crate::current::GOBLIN_PACKET_SEND;
        if m.read(base+rva,bytes.len())?!=bytes{return Err("Native pause motion sender changed".into())}
        type Send=unsafe extern "system" fn(u64,u64,*const u64);
        let send:Send=std::mem::transmute((base+rva)as usize);let mut sent=0;
        for actor in &party.actors{
            if actor.phase==asx_gameplay::Phase::Inactive{continue}
            let body=stop_body(actor.guid,actor.position,party.clock)?;
            for recipient in &party.actors{
                if recipient.slot==actor.slot||recipient.phase==asx_gameplay::Phase::Inactive{continue}
                // Rebind after each potentially reentrant send. Never target a reused slot.
                let current=self.world_party(m,game)?.ok_or("Pause run ended")?;
                if current.clock+0.001<party.clock||current.actors.first().map(|a|a.guid)!=party.actors.first().map(|a|a.guid){return Err("Pause run generation changed".into())}
                if !current.actors.iter().any(|a|a.slot==actor.slot&&a.guid==actor.guid&&a.phase!=asx_gameplay::Phase::Inactive)||!current.actors.iter().any(|a|a.slot==recipient.slot&&a.guid==recipient.guid&&a.phase!=asx_gameplay::Phase::Inactive){continue}
                let packet=[body.len()as u64,body.as_ptr()as u64,body.len()as u64,0xdd];
                send(game-16,u64::from(recipient.slot),packet.as_ptr());sent+=1;
            }
        }
        Ok(sent)
    }
}
#[cfg(test)]mod motion_tests{
    use super::*;
    #[test]fn stop_packet_matches_native_builder_and_rejects_invalid_inputs(){
        let body=stop_body(0x120034,[1.,2.,3.],4.).unwrap();let mut expected=vec![5,0x34,0x12];
        for v in [1f32,2.,3.]{expected.extend_from_slice(&v.to_le_bytes())}
        expected.extend_from_slice(&4000u32.to_le_bytes());expected.push(1);assert_eq!(body,expected);
        assert!(stop_body(0,[0.;3],0.).is_err());assert!(stop_body(1,[f32::NAN,0.,0.],0.).is_err());assert!(stop_body(1,[0.;3],-1.).is_err());
    }
}
