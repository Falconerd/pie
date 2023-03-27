mod pie;
use std::{env::args, fs::File, path::PathBuf};

pub use pie::{PixelFormat, DecodedPIE, EncodedPIE, Palette, read, write, encode, decode};

use png;

fn main() {
    let args: Vec<String> = args().collect();
    let decoder = png::Decoder::new(File::open(&args[1]).expect("Could not open PNG file"));
    let mut reader = decoder.read_info().expect("Could not read PNG file info");
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).expect("Could not read next frame of PNG");
    let bytes = &buf[..info.buffer_size()];

    let embed_palette = args.len() > 2 && &args[2] == "-e";

    let mut out_path = PathBuf::from(&args[1]);
    out_path.set_extension("pie");

    _ = pie::write(&out_path.to_owned().into_os_string().to_str().unwrap(), info.width as u16, info.height as u16, embed_palette, None, bytes.to_vec());
    println!("wrote: {:?}", &out_path.to_owned().into_os_string().to_str().unwrap());
}

