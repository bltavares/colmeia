use salsa20::stream_cipher::{NewStreamCipher, SyncStreamCipher};
use salsa20::XSalsa20;
use std::sync::{Arc, RwLock};

pub(crate) type SharedCipher = Arc<RwLock<Cipher>>;

pub struct Cipher {
    key: Vec<u8>,
    cipher: Option<XSalsa20>,
}

impl Cipher {
    pub fn new(key: Vec<u8>) -> Self {
        Self { key, cipher: None }
    }

    pub fn initialize(&mut self, nonce: &[u8]) {
        self.cipher = XSalsa20::new_var(&self.key, &nonce).ok();
    }

    pub(crate) fn try_apply(&mut self, buffer: &mut [u8]) {
        if let Some(ref mut cipher) = &mut self.cipher {
            cipher.apply_keystream(buffer);
        }
    }
}
