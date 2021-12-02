use coord::*;


pub struct View {
    pub radius: isize,
    pub diameter: isize,
    pub size: Pixel,
    pub origin: Pixel,
    pub positions: Vec<Hex>
}
impl View {
    pub fn new(radius: isize) -> View {
        let diameter = 2 * radius + 1;
        let size = HEX_SIZE * diameter + Pixel::new(HEX_SIZE.x / 3, 0);
        // Compute an iteration order for the view spaces
        // (top-down, then right-to-left, in a rectangle slightly larger than the window)
        let mut positions = vec!();
        for y in -2*(radius + 1) .. 2*(radius + 2) {
            let y = -y;
            for x in -(radius + 1) .. (radius + 2) {
                let x = -x;
                if (2*y + x).abs() <= 2*(radius + 1) {
                    positions.push(Hex::new(x, y));
                }
            }
        }
        View{
            radius:    radius,
            diameter:  diameter,
            size:      size,
            origin:    size / 2,
            positions: positions
        }
    }
}
