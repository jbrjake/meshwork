//! meshwork — thin binary shell; everything lives in the library
//! (`meshwork::cli` dispatches per DESIGN §6).

fn main() {
    std::process::exit(meshwork::cli::run());
}
