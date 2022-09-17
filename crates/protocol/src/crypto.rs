
use crypto::{
    hmac::Hmac, mac::Mac, rc4::Rc4, sha1::Sha1, symmetriccipher::SynchronousStreamCipher,
};

static DECRYPT_KEY: [u8; 16] = [
    0xC2, 0xB3, 0x72, 0x3C, 0xC6, 0xAE, 0xD9, 0xB5, 0x34, 0x3C, 0x53, 0xEE, 0x2F, 0x43, 0x67, 0xCE,
];
static ENCRYPT_KEY: [u8; 16] = [
    0xCC, 0x98, 0xAE, 0x04, 0xE8, 0x97, 0xEA, 0xCA, 0x12, 0xDD, 0xC0, 0x93, 0x42, 0x91, 0x53, 0x57,
];

pub struct HeaderCrypto {
    encrypt: Rc4,
    decrypt: Rc4,
}

impl HeaderCrypto {
    pub fn new(session_key: [u8; 40]) -> Self {
        let mut encrypt = Rc4::new({
            let mut hmac = Hmac::new(Sha1::new(), &ENCRYPT_KEY);
            hmac.input(&session_key);
            hmac.result().code()
        });

        let mut decrypt = Rc4::new({
            let mut hmac = Hmac::new(Sha1::new(), &DECRYPT_KEY);
            hmac.input(&session_key);
            hmac.result().code()
        });

        // drop first 1024 bytes (ARC4-drop1024)
        let sync_in = [0u8; 1024];
        let mut sync_out = [0u8; 1024];
        encrypt.process(&sync_in, &mut sync_out);
        decrypt.process(&sync_in, &mut sync_out);

        Self { encrypt, decrypt }
    }

    pub fn encrypt(&mut self, data: &mut [u8; 4]) {
        self.encrypt.process(&data.clone(), data)
    }

    pub fn decrypt(&mut self, data: &mut [u8; 6]) {
        self.decrypt.process(&data.clone(), data)
    }
}
