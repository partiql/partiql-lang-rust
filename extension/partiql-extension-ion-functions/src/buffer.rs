use flate2::read::GzDecoder;
use std::io::{BufReader, Read, Seek, SeekFrom};
use zstd::Decoder;

pub(crate) enum BufferType<'a, I: 'a + Read + 'static> {
    Gzip(BufReader<GzDecoder<I>>),
    Zstd(BufReader<Decoder<'a, BufReader<I>>>),
    Unknown(BufReader<I>),
}

pub(crate) fn infer_buffer_type<'a, I: 'a + Read + Seek + 'static>(
    mut reader: I,
) -> BufferType<'a, I> {
    let mut header: [u8; 4] = [0; 4];
    reader.read_exact(&mut header).expect("file header");
    reader.seek(SeekFrom::Start(0)).expect("file seek");

    if header.starts_with(&[0x1f, 0x8b]) {
        // Cf. https://datatracker.ietf.org/doc/html/rfc1952#page-6 section 2.3.1
        BufferType::Gzip(BufReader::new(flate2::read::GzDecoder::new(reader)))
    } else if header.starts_with(&[0x28, 0xB5, 0x2F, 0xFD]) {
        // Cf. https://datatracker.ietf.org/doc/rfc8478/ section 3.1.1
        BufferType::Zstd(BufReader::new(
            zstd::Decoder::new(reader).expect("zstd reader creation"),
        ))
    } else {
        BufferType::Unknown(BufReader::new(reader))
    }
}
