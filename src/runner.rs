use crate::pixel::Pix;

pub enum PixBuffer<'a> {
    Buf1(&'a [f32]),
    Buf3(&'a [Pix<f32>]),
}

pub trait Runner {
    fn applybuf<F>(&self, func: F)
    where F: FnMut(PixBuffer);
}

