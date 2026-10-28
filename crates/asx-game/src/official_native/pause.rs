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
        let adapter = SelectedAdapter::select(crate::official122::SHA256).unwrap();
        let layout = adapter.pause_layout();
        let keys = [
            0x1bf9c, 0x1bfcd, 0x346580, 0x346b4c, 0x346d29, 0x346cc9,
            0x1bbce, 0x3227b0, 0x31ddd0, 0x1bfc1, 0x1b6d4, 0x346ac3, 0x347403,
        ];
        assert_eq!(keys.map(|key| layout.rva(key)), adapter.pause_addresses());
    }
}
