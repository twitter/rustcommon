mod classic;
mod ironbow;

pub(crate) use classic::CLASSIC;
pub(crate) use ironbow::IRONBOW;

#[derive(Copy, Clone)]
pub enum Palette {
    Classic,
    Ironbow,
}
