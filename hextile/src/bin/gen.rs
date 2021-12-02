extern crate hextile;

use self::hextile::system::unsafe_cur_dir;
use self::hextile::gen::generate_assets;

pub fn main() {
    generate_assets(&mut unsafe_cur_dir());
}
