use std::mem;
use std::slice;
use std::ops::{Deref, DerefMut};

use libc::c_int;
use ffi::*;
use ::Rational;
use ::util::format;
use ::util::chroma;
use ::picture;
use ::color;
use super::Frame;

#[derive(PartialEq, Eq)]
pub struct Video(Frame);

impl Video {
	pub unsafe fn wrap(ptr: *mut AVFrame) -> Self {
		Video(Frame::wrap(ptr))
	}

	pub unsafe fn alloc(&mut self, format: format::Pixel, width: u32, height: u32) {
		self.set_format(format);
		self.set_width(width);
		self.set_height(height);

		av_frame_get_buffer(self.as_mut_ptr(), 1);
	}
}

impl Video {
	pub fn empty() -> Self {
		unsafe {
			Video(Frame::empty())
		}
	}

	pub fn new(format: format::Pixel, width: u32, height: u32) -> Self {
		unsafe {
			let mut frame = Video::empty();
			frame.alloc(format, width, height);

			frame
		}
	}

	pub fn format(&self) -> format::Pixel {
		unsafe {
			if (*self.as_ptr()).format == -1 {
				format::Pixel::None
			}
			else {
				format::Pixel::from(mem::transmute::<_, AVPixelFormat>(((*self.as_ptr()).format)))
			}
		}
	}

	pub fn set_format(&mut self, value: format::Pixel) {
		unsafe {
			(*self.as_mut_ptr()).format = mem::transmute::<AVPixelFormat, c_int>(value.into());
		}
	}

	pub fn kind(&self) -> picture::Type {
		unsafe {
			picture::Type::from((*self.as_ptr()).pict_type)
		}
	}

	pub fn is_interlaced(&self) -> bool {
		unsafe {
			(*self.as_ptr()).interlaced_frame != 0
		}
	}

	pub fn is_top_first(&self) -> bool {
		unsafe {
			(*self.as_ptr()).top_field_first != 0
		}
	}

	pub fn has_palette_changed(&self) -> bool {
		unsafe {
			(*self.as_ptr()).palette_has_changed != 0
		}
	}

	pub fn width(&self) -> u32 {
		unsafe {
			(*self.as_ptr()).width as u32
		}
	}

	pub fn set_width(&mut self, value: u32) {
		unsafe {
			(*self.as_mut_ptr()).width = value as c_int;
		}
	}

	pub fn height(&self) -> u32 {
		unsafe {
			(*self.as_ptr()).height as u32
		}
	}

	pub fn set_height(&mut self, value: u32) {
		unsafe {
			(*self.as_mut_ptr()).height = value as c_int;
		}
	}

	pub fn color_space(&self) -> color::Space {
		unsafe {
			color::Space::from(av_frame_get_colorspace(self.as_ptr()))
		}
	}

	pub fn set_color_space(&mut self, value: color::Space) {
		unsafe {
			av_frame_set_colorspace(self.as_mut_ptr(), value.into());
		}
	}

	pub fn color_range(&self) -> color::Range {
		unsafe {
			color::Range::from(av_frame_get_color_range(self.as_ptr()))
		}
	}

	pub fn set_color_range(&mut self, value: color::Range) {
		unsafe {
			av_frame_set_color_range(self.as_mut_ptr(), value.into());
		}
	}

	pub fn color_primaries(&self) -> color::Primaries {
		unsafe {
			color::Primaries::from((*self.as_ptr()).color_primaries)
		}
	}

	pub fn set_color_primaries(&mut self, value: color::Primaries) {
		unsafe {
			(*self.as_mut_ptr()).color_primaries = value.into();
		}
	}

	pub fn color_transfer_characteristic(&self) -> color::TransferCharacteristic {
		unsafe {
			color::TransferCharacteristic::from((*self.as_ptr()).color_trc)
		}
	}

	pub fn set_color_transfer_characteristic(&mut self, value: color::TransferCharacteristic) {
		unsafe {
			(*self.as_mut_ptr()).color_trc = value.into();
		}
	}

	pub fn chroma_location(&self) -> chroma::Location {
		unsafe {
			chroma::Location::from((*self.as_ptr()).chroma_location)
		}
	}

	pub fn aspect_ratio(&self) -> Rational {
		unsafe {
			Rational::from((*self.as_ptr()).sample_aspect_ratio)
		}
	}

	pub fn coded_number(&self) -> usize {
		unsafe {
			(*self.as_ptr()).coded_picture_number as usize
		}
	}

	pub fn display_number(&self) -> usize {
		unsafe {
			(*self.as_ptr()).display_picture_number as usize
		}
	}

	pub fn repeat(&self) -> f64 {
		unsafe {
			(*self.as_ptr()).repeat_pict as f64
		}
	}

	pub fn planes(&self) -> usize {
		for i in 0 .. 8 {
			unsafe {
				if (*self.as_ptr()).linesize[i] == 0 {
					return i;
				}
			}
		}

		8
	}

	pub fn plane<T: Component>(&self, index: usize) -> &[T] {
		if index >= self.planes() {
			panic!("out of bounds");
		}

		if !<T as Component>::is_valid(self.format()) {
			panic!("unsupported type");
		}

		unsafe {
			slice::from_raw_parts(
				mem::transmute((*self.as_ptr()).data[index]),
				(*self.as_ptr()).linesize[index] as usize * self.height() as usize / mem::size_of::<T>())
		}
	}

	pub fn plane_mut<T: Component>(&mut self, index: usize) -> &mut[T] {
		if index >= self.planes() {
			panic!("out of bounds");
		}

		if !<T as Component>::is_valid(self.format()) {
			panic!("unsupported type");
		}

		unsafe {
			slice::from_raw_parts_mut(
				mem::transmute((*self.as_mut_ptr()).data[index]),
				(*self.as_ptr()).linesize[index] as usize * self.height() as usize / mem::size_of::<T>())
		}
	}

	pub fn data(&self) -> Vec<&[u8]> {
		let mut result = Vec::new();

		unsafe {
			for (i, length) in (*self.as_ptr()).linesize.iter().take_while(|l| **l > 0).enumerate() {
				result.push(slice::from_raw_parts(
					(*self.as_ptr()).data[i],
					*length as usize * self.height() as usize));
			}
		}

		result
	}

	pub fn data_mut(&mut self) -> Vec<&mut [u8]> {
		let mut result = Vec::new();

		unsafe {
			for (i, length) in (*self.as_ptr()).linesize.iter().take_while(|l| **l > 0).enumerate() {
				result.push(slice::from_raw_parts_mut(
					(*self.as_mut_ptr()).data[i],
					*length as usize * self.height() as usize));
			}
		}

		result
	}
}

impl Deref for Video {
	type Target = Frame;

	fn deref(&self) -> &Frame {
		&self.0
	}
}

impl DerefMut for Video {
	fn deref_mut(&mut self) -> &mut Frame {
		&mut self.0
	}
}

impl Clone for Video {
	fn clone(&self) -> Self {
		let mut cloned = Video::new(self.format(), self.width(), self.height());
		cloned.clone_from(self);

		cloned
	}

	fn clone_from(&mut self, source: &Self) {
		unsafe {
			av_frame_copy(self.as_mut_ptr(), source.as_ptr());
			av_frame_copy_props(self.as_mut_ptr(), source.as_ptr());
		}
	}
}

pub trait Component {
	fn is_valid(format: format::Pixel) -> bool;
}

#[cfg(feature = "image")]
impl Component for ::image::Luma<u8> {
	fn is_valid(format: format::Pixel) -> bool {
		format == format::Pixel::GRAY8
	}
}

#[cfg(feature = "image")]
impl Component for ::image::Rgb<u8> {
	fn is_valid(format: format::Pixel) -> bool {
		format == format::Pixel::RGB24
	}
}

#[cfg(feature = "image")]
impl Component for ::image::Rgba<u8> {
	fn is_valid(format: format::Pixel) -> bool {
		format == format::Pixel::RGBA
	}
}

impl Component for [u8; 3] {
	fn is_valid(format: format::Pixel) -> bool {
		format == format::Pixel::RGB24 || format == format::Pixel::BGR24
	}
}

impl Component for (u8, u8, u8) {
	fn is_valid(format: format::Pixel) -> bool {
		format == format::Pixel::RGB24 || format == format::Pixel::BGR24
	}
}

impl Component for [u8; 4] {
	fn is_valid(format: format::Pixel) -> bool {
		format == format::Pixel::RGBA || format == format::Pixel::BGRA ||
		format == format::Pixel::ARGB || format == format::Pixel::ABGR ||
		format == format::Pixel::RGBZ || format == format::Pixel::BGRZ ||
		format == format::Pixel::ZRGB || format == format::Pixel::ZBGR
	}
}

impl Component for (u8, u8, u8, u8) {
	fn is_valid(format: format::Pixel) -> bool {
		format == format::Pixel::RGBA || format == format::Pixel::BGRA ||
		format == format::Pixel::ARGB || format == format::Pixel::ABGR ||
		format == format::Pixel::RGBZ || format == format::Pixel::BGRZ ||
		format == format::Pixel::ZRGB || format == format::Pixel::ZBGR
	}
}
