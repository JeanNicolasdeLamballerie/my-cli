use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use base64::{self, Engine};
use chacha20poly1305::{aead::Aead, AeadCore, ChaCha20Poly1305, KeyInit, Nonce};
use diesel::SqliteConnection;
use std::str;

use crate::{
    database::{self, establish_connection, CryptoFilterType},
    models::{CryptoData, MasterUser},
};
use egui;

#[cfg(not(target_os = "windows"))]
const NL: &str = "\n";

#[cfg(target_os = "windows")]
const NL: &str = "\r\n";

pub fn show_password(pw: &str) -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| {
            // This gives us image support:
            // egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::new(MyEguiApp::new(cc, pw.to_string())))
        }),
    )
}
#[derive(Default)]
struct MyEguiApp {
    pw: String,
    pw_string: String,
}

impl MyEguiApp {
    fn new(_cc: &eframe::CreationContext<'_>, pw: String) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        let mut pw_string = String::new();
        for _ in 0..pw.len() {
            pw_string.push('*');
        }
        Self { pw, pw_string }
    }
}
impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello World!");
            ui.label("Password");
            if ui.button("Show").clicked() {
                self.pw_string = self.pw.to_string();
            };

            ui.label(&self.pw_string);
        });
    }
}

pub fn requires_password(conn: &mut SqliteConnection) -> [u8; 32] {
    let password = rpassword::prompt_password("Your password: ").unwrap();
    let key = verify_master_password(conn, password);
    println!("your key : {:?}", key);
    key
}

pub fn hidden_user_input(depth: u8) -> String {
    let data = rpassword::prompt_password("Enter password/data to encrypt: ").unwrap();
    let verify_data = rpassword::prompt_password("Verify password/data to encrypt: ").unwrap();
    if data != verify_data {
        if depth > 5 {
            panic!("seriously, what are you doing ? Make sure both of the data entered are correct...:D ");
        }
        println!("passwords didnt' match. Try again.");
        return hidden_user_input(depth + 1);
    }
    return data;
}
fn make_master_pw(depth: u8) -> String {
    let password = rpassword::prompt_password("Enter master password: ").unwrap();
    let verify_password = rpassword::prompt_password("Verify master password: ").unwrap();
    if password != verify_password {
        if depth > 5 {
            panic!("seriously, what are you doing ? Make sure both passwords are correct... :D ");
        }
        println!("passwords didnt' match. Try again.");
        return make_master_pw(depth + 1);
    }
    return password;
}

fn create_master_password() -> MasterUser {
    use colored::Colorize;

    println!(
        "{}",
        "---------You've just run an authenticated command without a user registered.---------"
            .blue()
            .bold()
    );
    println!(
        "{}",
        "---------Starting the password setting process.---------"
            .blue()
            .bold()
    );
    println!("{}", "---------------------------------".red().bold());
    println!("{}", " BE CAREFUL ".red().bold().underline());
    println!("{}", "---------------------------------".red().bold());
    println!("This password will only be created once. It is a master password, and needs to be secure; it cannot be retrieved. Don't forget it, and no bad passwords !");
    println!(
        "{}",
        "--------------Setting your master password-----------------"
            .blue()
            .bold()
    );
    let password = make_master_pw(0);

    let salt = SaltString::generate(&mut OsRng);
    // TODO Argon2 params
    // Argon2 with default params (Argon2id v19)
    let argon2 = Argon2::default();

    // Hash password to PHC string ($argon2id$v=19$...)
    let password_hash = match argon2.hash_password(&password.as_bytes(), &salt) {
        Ok(hashed) => hashed.to_string(),
        Err(error) => {
            panic!("An error occured while hashing the password. See below : {NL} {error}")
        }
    };
    store_master_password(&password_hash)
}

fn verify_master_password(conn: &mut SqliteConnection, cipher_pass: String) -> [u8; 32] {
    let master_user = retrieve_master_password(conn);
    let password_hash = master_user.hash;
    // Verify password against PHC string.
    //
    // NOTE: hash params from `parsed_hash` are used instead of what is configured in the
    // `Argon2` instance.
    let parsed_hash = match PasswordHash::new(&password_hash) {
        Ok(pw) => pw,
        Err(error) => {
            panic!("An error occured while hashing. See below : {NL} {error}")
        }
    };
    //TODO change assert ?
    assert!(Argon2::default()
        .verify_password(&cipher_pass.as_bytes(), &parsed_hash)
        .is_ok());

    let mut key_val: [u8; 32] = [0; 32];
    //TODO CHECK WHICH SALT TO USE
    let salt: [u8; 16] = [1; 16];
    match Argon2::default().hash_password_into(cipher_pass.as_bytes(), &salt, &mut key_val) {
        Ok(()) => (),
        Err(err) => panic!("error : {NL}{err}"),
    };
    key_val
}

fn store_master_password(phc_string: &str) -> MasterUser {
    let mut conn = database::establish_connection();
    database::create_master_user(&mut conn, phc_string)
}

fn retrieve_master_password(conn: &mut SqliteConnection) -> MasterUser {
    let pw = database::fetch_master_user(conn);
    return match pw {
        Some(hash) => hash,
        None => create_master_password(),
    };
}

pub fn decrypt(data: &[u8], key: &[u8; 32], nonce: Nonce) -> String {
    let cipher = ChaCha20Poly1305::new(key.into());
    let plaintext_password_option = cipher.decrypt(&nonce, data);
    match plaintext_password_option {
        Ok(plaintext_password) => {
            let s = match str::from_utf8(&plaintext_password) {
                Ok(v) => v,
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            };
            s.to_string()
            // Do what you need to with the decrypted password
        }
        Err(e) => {
            println!("{e}");
            // Oh no-an attacker tampered with the encrypted password
            panic!("An attacker might have tempered with the encrypted passwords file. Consider taking appropriate actions.")
        }
    }
}

// fn retrieve_encrypted(key: &[u8; 32]) -> String {
//     let encrypted_password = get_encrypted_password();
//     let nonce = read_stored_nonce();

// }
pub fn nonce_from_db_string(nonce_string: &str) -> Nonce {
    let mut i: usize = 0;
    let mut nonce: [u8; 12] = [0; 12];
    for string_u8 in nonce_string.split(",") {
        if i >= 12 {
            panic!("This array should not be longer than 32 bytes...");
        }
        let nonce_chunk = string_u8.parse::<u8>().unwrap();
        nonce[i] = nonce_chunk;
        i += 1;
    }
    nonce.into()
}
pub trait DbReady {
    fn to_db_string(&self) -> String;
}
impl DbReady for Nonce {
    fn to_db_string(&self) -> String {
        let mut db_string = String::new();
        for value in self.iter() {
            db_string += &value.to_string();
            db_string += ",";
        }
        db_string.pop();
        return db_string;
    }
}

pub fn encrypt(data: &str, key_val: &[u8; 32]) -> (Vec<u8>, Nonce) {
    // Nonce should be different for every encryption
    let cipher = ChaCha20Poly1305::new(key_val.into());
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng); // 96-bits; unique per message
    let encrypted = match cipher.encrypt(&nonce, data.as_bytes()) {
        Ok(encrypted_data) => encrypted_data,
        Err(err) => panic!("couldn't encrypt the data, see error  : {err}"),
    };
    (encrypted, nonce)
}

pub fn store_encrypted(data: &str, host: &str, key: &[u8; 32]) -> CryptoData {
    let (encrypted, nonce) = encrypt(data, key);
    let n = nonce.to_db_string();
    let b64_encrypted = base64::prelude::BASE64_STANDARD.encode(&encrypted);
    let mut conn = establish_connection();
    database::create_crypto(&mut conn, &b64_encrypted, &n, &host)
}
pub fn retrieve_encrypted(key: &[u8; 32], filter: CryptoFilterType) -> String {
    let mut conn = database::establish_connection();
    let crypto = database::fetch_crypto(&mut conn, filter);
    let cleartext_encrypted = decrypt(
        &base64::prelude::BASE64_STANDARD
            .decode(&crypto[0].encrypted)
            .unwrap(),
        key,
        nonce_from_db_string(&crypto[0].nonce),
    );
    cleartext_encrypted
}
