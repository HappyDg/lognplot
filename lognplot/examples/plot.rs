/// Demonstration of the plot usage!
use std::fs::File;

use lognplot::chart::plot;
use lognplot::geometry::Size;
use lognplot::render::SvgOutput;

fn main() {
    simple_logger::init().unwrap();

    let x = vec![1.0, 2.0, 3.0, 4.0, 5.0, 8.0];
    let y = vec![9.0, 2.2, 5.5, 2.2, 1.2, 1.7];

    let mut buffer = File::create("plot.svg").unwrap();
    let mut canvas = SvgOutput::new(&mut buffer);

    let size = Size::new(1000.0, 1000.0);
    plot(&mut canvas, x, y, size);
}
