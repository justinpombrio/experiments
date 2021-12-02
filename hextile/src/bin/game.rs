extern crate hextile;

use hextile::system::unsafe_cur_dir;
use hextile::game;

fn main() {
    game::play(&unsafe_cur_dir())
}
