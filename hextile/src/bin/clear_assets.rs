extern crate hextile;

use self::hextile::system::unsafe_cur_dir;
use self::hextile::gen::clear_assets;

pub fn main() {
    clear_assets(&mut unsafe_cur_dir());
}
