use self_update::update_self_exe;
use pretty_env_logger;

const UPDATE: &'static [u8] = include_bytes!("../update");

fn main() {
    pretty_env_logger::init();
    println!("Hello, world!");
    println!("I am now going to update...");
    update_self_exe(UPDATE).unwrap();
    println!("This should never be printed!");
}
