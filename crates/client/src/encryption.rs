use aes_gcm::{
    AeadCore, Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, OsRng},
};
use base64::engine::general_purpose;
use base64::{Engine, engine::general_purpose::STANDARD};
use keyring::Entry;
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;
use zeroize::Zeroize;

const ITERATIONS: u32 = 100_000;
const KEY_LEN: usize = 32; // 256 bits
const SALT: &str = "photon-hq/enva";
const SERVICE: &str = "codes.photon.enva";

pub fn save_pwd(repo_url: &str, password: &str) {
    let (owner, repo_name) = shared::parse_github_repo(repo_url).expect("Invalid repo URL");
    
    let key = Vec::from(derive_key(&owner, &repo_name, password));
    save_derived_key(&owner, &repo_name, key).expect("Failed to save key to keychain");
}

pub fn encrypt_string(repo_url: &str, plaintext: &str) -> String {
    let (owner, repo_name) = shared::parse_github_repo(repo_url).expect("Invalid repo URL");
    
    let (ciphertext, nonce) = encrypt(&load_derived_key(&owner, &repo_name).expect("Failed to load key from keychain"), plaintext.as_bytes());

    // combine nonce + ciphertext
    let mut output = Vec::new();
    output.extend_from_slice(&nonce);
    output.extend_from_slice(&ciphertext);

    general_purpose::STANDARD.encode(output)
}

pub fn decrypt_string(repo_url: &str, encrypted_b64: &str) -> String {
    let (owner, repo_name) = shared::parse_github_repo(repo_url).expect("Invalid repo URL");
    
    let data = general_purpose::STANDARD.decode(encrypted_b64).unwrap();

    let (nonce, ciphertext) = data.split_at(12);

    let plaintext = decrypt(&load_derived_key(&owner, &repo_name).expect("Failed to load key from keychain"), ciphertext, nonce.try_into().unwrap());

    String::from_utf8(plaintext).unwrap()
}

fn derive_key(owner: &str, repo_name: &str, password: &str) -> [u8; KEY_LEN] {
    let effective_salt = format!("{owner}:{repo_name}:{SALT}");
    let mut key = [0u8; KEY_LEN];

    pbkdf2_hmac::<Sha256>(
        password.as_bytes(),
        effective_salt.as_bytes(),
        ITERATIONS,
        &mut key,
    );

    key
}

fn encrypt(key: &[u8], plaintext: &[u8]) -> (Vec<u8>, [u8; 12]) {
    let cipher = Aes256Gcm::new_from_slice(key).unwrap();

    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, plaintext).unwrap();

    (ciphertext, nonce.into())
}

fn decrypt(key: &[u8], ciphertext: &[u8], nonce: &[u8; 12]) -> Vec<u8> {
    let cipher = Aes256Gcm::new_from_slice(key).unwrap();
    cipher
        .decrypt(Nonce::from_slice(nonce), ciphertext)
        .unwrap()
}

fn save_derived_key(owner: &str, repo_name: &str, mut key: Vec<u8>) -> Result<(), keyring::Error> {
    let entry = Entry::new(SERVICE, format!("{owner}:{repo_name}").as_str())?;

    let encoded = STANDARD.encode(&key);
    entry.set_password(&encoded)?;

    // wipe key from memory after storing
    key.zeroize();

    Ok(())
}

fn load_derived_key(owner: &str, repo_name: &str) -> Result<Vec<u8>, keyring::Error> {
    let entry = Entry::new(SERVICE, format!("{owner}:{repo_name}").as_str())?;
    let encoded = entry.get_password()?;

    let key = STANDARD
        .decode(encoded)
        .expect("invalid key material in keychain");

    Ok(key)
}
