#[cfg(windows)]
extern crate windres;

fn main() {
    #[cfg(windows)]
    {
        windres::Build::new().compile("version.rc").unwrap();
    }
}
