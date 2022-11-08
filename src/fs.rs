use actix_web::web::Bytes;
use std::{ fs::File, io::Write };

pub fn write_file(
  bytes: &Bytes,
  path: String
) -> Result<String, std::io::Error> {
  let mut file = File::create(path).unwrap();
  match file.write_all(&bytes) {
    Ok(_) => Ok("".to_owned()),
    Err(err) => Err(err),
  }
}

pub fn mkdir(dir_path: String) -> anyhow::Result<()> {
  std::fs::create_dir_all(dir_path)?;
  Ok(())
}