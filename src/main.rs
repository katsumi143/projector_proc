use clap::Parser;
use image::{ ImageReader, Rgba, RgbaImage };
use std::path::Path;

#[derive(Debug, Parser)]
struct Cli {
	#[arg(short, long)]
	input: String,
	
	#[arg(short, long)]
	destination: String,
	
	#[arg(short, long, default_value = "false")]
	output_processed_source: bool
}

pub const CHANNEL_LETTERS: [char; 3] = ['R', 'G', 'B'];

fn truncate_bit_depth(value: u8, bit_depth: u8) -> u8 {
	value >> (8 - bit_depth)
}

fn clamp_spread_range(value: u8, bit_depth: u8) -> u8 {
	(255 * truncate_bit_depth(value, bit_depth) as u32 / ((1 << bit_depth) - 1) as u32) as u8
}

fn main() {
	let args = Cli::parse();
	let image = ImageReader::open(&args.input)
		.unwrap()
		.decode()
		.unwrap();
	
	let width = image.width();
	let height = image.height();
	
	let mut buffer = image.into_rgba8();
	let pixels = buffer.enumerate_pixels_mut();
	
	let depth = 1;
	for pixel in pixels {
		let data = pixel.2;
		data[0] = clamp_spread_range(data[0], depth);
		data[1] = clamp_spread_range(data[1], depth);
		data[2] = clamp_spread_range(data[2], depth);
	}
	
	let file_name = Path::new(&args.input).file_name().unwrap().to_str().unwrap().split('.').next().unwrap();
	let destination = Path::new(&args.destination);
	std::fs::create_dir_all(destination)
		.unwrap();
	
	for i in 0..3 {
		let mut buffer_1 = RgbaImage::new(width, height);
		for pixel in buffer.enumerate_pixels() {
			if pixel.2.0[i] == 0 {
				buffer_1.put_pixel(pixel.0, pixel.1, Rgba([0, 0, 0, 255]));
			}
		}
		
		buffer_1
			.save(destination.join(format!("{file_name}_channel{}.png", CHANNEL_LETTERS[i])))
			.unwrap();
	}
	
	if args.output_processed_source {
		buffer
			.save(destination.join(format!("{file_name}_processed.png")))
			.unwrap();
	}
}