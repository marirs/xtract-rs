use xtract::{get_total_files, Result};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("{} <file>", args[0]);
        return Ok(());
    }
    let file_name = &args[1];
    eprintln!("{}", get_total_files(file_name)?);

    Ok(())
}
