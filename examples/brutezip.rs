use xtract::{from_zipfile, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("{} <file> <password1> [<password2>...]", args[0]);
        return Ok(());
    }
    let file_name = &args[1];
    let passwords = &args[2..];
    eprintln!("--- Summary");
    eprintln!("File: {}", file_name);
    eprintln!("Passwords: {:?}", passwords);

    let zip = from_zipfile(file_name.to_string(), Some(passwords.to_vec())).await?;
    if zip.is_empty() {
        eprintln!("[!] can`t decrypt");
    } else {
        eprintln!(
            "[*] decrypted {} files with pass {}",
            zip.len(),
            if let Some(pass) = &zip[0].zip_password{pass}else{"NONE"}
        );
    }
    Ok(())
}
