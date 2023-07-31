use gamezap::run;
use pollster::{self, block_on};

mod camera;
mod engine;
mod gamezap;
mod model;
mod resources;
mod texture;
mod utils;

fn main() {
    block_on(run());
}
