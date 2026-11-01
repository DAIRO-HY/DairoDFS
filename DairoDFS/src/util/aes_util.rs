use crate::application;
use aes::{Aes128, Aes192, Aes256};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE;
use cfb_mode::cipher::{AsyncStreamCipher, KeyIvInit};
use rand::distr::{Alphanumeric, SampleString};
use std::path::PathBuf;
use std::sync::LazyLock;

/// AES的块大小,也是初始化向量的长度
const BLOCK_SIZE: usize = 16;

/// AES密钥文件(与老版本一致)
static KEY_PATH: LazyLock<PathBuf> = LazyLock::new(|| PathBuf::from(&application::ARGS.data_dir).join("AES_KEY"));

/// AES密钥,文件不存在时生成一个16位的随机密钥并保存,以后一直使用这个密钥
static KEY: LazyLock<Vec<u8>> = LazyLock::new(|| {
    if let Ok(data) = std::fs::read(KEY_PATH.as_path()) {
        if matches!(data.len(), 16 | 24 | 32) {
            return data;
        }
        eprintln!("-->AES密钥文件:{}长度不正确", KEY_PATH.display());
        std::process::exit(1);
    }
    let key = Alphanumeric.sample_string(&mut rand::rng(), 16).into_bytes();
    if let Err(e) = std::fs::write(KEY_PATH.as_path(), &key) {
        eprintln!("-->创建AES密钥文件:{}失败: {}", KEY_PATH.display(), e);
        std::process::exit(1);
    }
    key
});

/// 获取AES密钥
pub fn key() -> &'static [u8] {
    &KEY
}

/// 加密: 随机初始化向量 + AES-CFB加密后的数据,再做URL安全的base64编码
pub fn encrypt(plaintext: &str) -> String {
    let mut data = vec![0u8; BLOCK_SIZE];
    rand::fill(&mut data[..]);
    let iv = data.clone();
    data.extend_from_slice(plaintext.as_bytes());
    let buf = &mut data[BLOCK_SIZE..];
    let key = key();
    match key.len() {
        16 => cfb_mode::Encryptor::<Aes128>::new_from_slices(key, &iv).unwrap().encrypt(buf),
        24 => cfb_mode::Encryptor::<Aes192>::new_from_slices(key, &iv).unwrap().encrypt(buf),
        _ => cfb_mode::Encryptor::<Aes256>::new_from_slices(key, &iv).unwrap().encrypt(buf),
    }
    URL_SAFE.encode(data)
}

/// 解密,格式不正确时返回None
pub fn decrypt(encrypted: &str) -> Option<String> {
    let data = URL_SAFE.decode(encrypted).ok()?;
    if data.len() < BLOCK_SIZE {
        return None;
    }
    let (iv, ciphertext) = data.split_at(BLOCK_SIZE);
    let mut buf = ciphertext.to_vec();
    let key = key();
    match key.len() {
        16 => cfb_mode::Decryptor::<Aes128>::new_from_slices(key, iv).ok()?.decrypt(&mut buf),
        24 => cfb_mode::Decryptor::<Aes192>::new_from_slices(key, iv).ok()?.decrypt(&mut buf),
        _ => cfb_mode::Decryptor::<Aes256>::new_from_slices(key, iv).ok()?.decrypt(&mut buf),
    }
    String::from_utf8(buf).ok()
}
