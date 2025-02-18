use clap::{ Parser, ValueEnum };
use image::{ ImageReader, Rgba, RgbaImage };
use std::path::Path;

#[derive(Parser)]
#[command(version)]
struct Cli {
	#[arg(short, long)]
	input: String,
	
	#[arg(short, long)]
	destination: String,
	
	#[arg(short, long, default_value = "split", value_enum)]
	mode: Mode,
	
	#[arg(short, long, default_value = "false")]
	output_processed_source: bool
}

#[derive(Clone, ValueEnum)]
enum Mode {
	Split,
	MergeX,
	MergeY
}

impl Mode {
	fn direction(&self) -> Option<[u32; 2]> {
		match self {
			Self::Split => None,
			Self::MergeX => Some([1, 0]),
			Self::MergeY => Some([0, 1])
		}
	}
	
	fn name(&self) -> &str {
		match self {
			Self::Split => "split",
			Self::MergeX => "mergedX",
			Self::MergeY => "mergedY"
		}
	}
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
	let mut channel_buffers: [RgbaImage; 3] = [
		RgbaImage::new(width, height),
		RgbaImage::new(width, height),
		RgbaImage::new(width, height)
	];
	
	for pixel in buffer.enumerate_pixels_mut() {
		let data = pixel.2;
		data[0] = clamp_spread_range(data[0], 1);
		data[1] = clamp_spread_range(data[1], 1);
		data[2] = clamp_spread_range(data[2], 1);
		for i in 0..3 {
			if data[i] == 0 {
				channel_buffers[i].put_pixel(pixel.0, pixel.1, Rgba([0, 0, 0, 255]));
			}
		}
	}
	
	let file_name = Path::new(&args.input).file_name().unwrap().to_str().unwrap().split('.').next().unwrap();
	let destination = Path::new(&args.destination);
	std::fs::create_dir_all(destination)
		.unwrap();
	
	if let Some(direction) = args.mode.direction() {
		let mut merged_buffer = RgbaImage::new(width * (direction[0] * 3).max(1), height * (direction[1] * 3).max(1));
		for i in 0..3 {
			let offset_x = i as u32 * width * direction[0];
			let offset_y = i as u32 * height * direction[1];
			for (x, y, pixel) in channel_buffers[i].enumerate_pixels() {
				merged_buffer.put_pixel(offset_x + x, offset_y + y, *pixel);
			}
		}
		
		merged_buffer
			.save(destination.join(format!("{file_name}_{}.png", args.mode.name())))
			.unwrap();
	} else {
		for i in 0..3 {
			channel_buffers[i]
				.save(destination.join(format!("{file_name}_channel{}.png", CHANNEL_LETTERS[i])))
				.unwrap();
		}
	}
	
	if args.output_processed_source {
		buffer
			.save(destination.join(format!("{file_name}_processed.png")))
			.unwrap();
	}
}