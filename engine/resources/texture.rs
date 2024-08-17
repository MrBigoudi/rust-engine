use std::any::Any;

pub trait Texture {
    fn get_id(&self) -> u32;

    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;

    fn get_nb_channels(&self) -> u8;

    fn has_transparency(&self) -> bool;

    fn get_generation(&self) -> Option<u32>;
    fn as_any(&self) -> &dyn Any;

    fn clone_box(&self) -> Box<dyn Texture>;
}

pub struct TextureCreatorParameters<'a> {
    pub name: &'a str,
    pub auto_release: bool,
    pub width: u32,
    pub height: u32,
    pub nb_channels: u8,
    pub pixels: &'a [u8],
    pub has_transparency: bool,
    pub is_default: bool,
}
