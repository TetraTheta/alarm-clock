use std::io;

fn main() -> io::Result<()> {
  println!("cargo:rerun-if-changed=resource/app.manifest");
  println!("cargo:rerun-if-changed=resource/main.ico");

  if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
    winresource::WindowsResource::new().set_icon("resource/main.ico").set_manifest_file("resource/app.manifest").compile()?;
  }

  Ok(())
}
