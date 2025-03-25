use argon2rs::argon2i_simple;

use super::model::Account;

pub fn make_salt() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789)(*&^%$#@!~";
    const PASSWORD_LEN: usize = 128;
    let mut rng = rand::rng();

    let password: String = (0..PASSWORD_LEN)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    password
}

pub fn make_hash(password: &str, salt: &str) -> [u8; argon2rs::defaults::LENGTH] {
    argon2i_simple(password, salt)
}

pub fn verify(account: &Account, password: &str) -> bool {
    let Account { hash, salt, .. } = account;

    make_hash(password, salt) == hash.as_ref()
}
