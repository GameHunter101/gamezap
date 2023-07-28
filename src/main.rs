use engine::Engine;
use gamezap::run;
use pollster::{self, block_on};
use sdl2;
use wgpu;

mod engine;
mod gamezap;
mod utils;

fn main() {
    block_on( run());
}
