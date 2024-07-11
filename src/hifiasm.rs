use std::{
    fs::File,
    io::{BufReader, Read, Seek},
};

pub fn read_seq_info(path: &str) -> (u64, Vec<u32>, Vec<Vec<u8>>, Vec<Vec<usize>>) {
    let mut buffer = [0u8; 44];
    let mut reader = BufReader::new(File::open(path).unwrap());

    reader
        .read_exact(&mut buffer)
        .expect("Failed to parse the ec bin file");
    let _name_index_size = u64::from_ne_bytes(buffer[12..20].try_into().unwrap()) as usize;
    let reads_count = u64::from_ne_bytes(buffer[20..28].try_into().unwrap()) as usize;
    let total_name_length = u64::from_ne_bytes(buffer[36..].try_into().unwrap()) as usize;

    let mut n_sites = Vec::with_capacity(reads_count);
    for _ in 0..reads_count {
        reader
            .read_exact(&mut buffer[..8])
            .expect("Failed to parse the ec bin file");
        let len = u64::from_ne_bytes(buffer[..8].try_into().unwrap()) as usize;
        if len != 0 {
            let mut poses = Vec::with_capacity(len);
            for _ in 0..len {
                reader
                    .read_exact(&mut buffer[..8])
                    .expect("Failed to parse the ec bin file");
                poses.push(u64::from_ne_bytes(buffer[..8].try_into().unwrap()) as usize);
            }
            n_sites.push(poses);
        } else {
            n_sites.push(Vec::new());
        }
    }

    let mut read_lens = Vec::with_capacity(reads_count);
    for _i in 0..reads_count {
        reader
            .read_exact(&mut buffer[..8])
            .expect("Failed to parse the ec bin file");
        read_lens.push(u64::from_ne_bytes(buffer[..8].try_into().unwrap()) as u32);
    }
    let offset = reader.stream_position().unwrap();

    let read_seperate = read_lens.iter().map(|x| *x as u64 / 4 + 1).sum::<u64>();
    reader
        .seek(std::io::SeekFrom::Current(read_seperate as i64))
        .expect("Failed to seek the ec bin file");

    let mut read_names = vec![0; total_name_length];
    reader
        .read_exact(&mut read_names[..])
        .expect("Failed to parse the ec bin file");

    let mut read_name_index = Vec::with_capacity(reads_count + 1);
    for _ in 0..=reads_count {
        reader
            .read_exact(&mut buffer[..8])
            .expect("Failed to parse the ec bin file");
        read_name_index.push(u64::from_ne_bytes(buffer[..8].try_into().unwrap()) as usize);
    }

    let mut new_read_names = Vec::with_capacity(reads_count);
    for i in 0..reads_count {
        new_read_names.push(read_names[read_name_index[i]..read_name_index[i + 1]].to_owned());
    }

    (offset, read_lens, new_read_names, n_sites)
    // for i in 0..reads_count{
    // 	println!("{}\t{}", std::str::from_utf8(&new_read_names[i]).unwrap(), read_lens[i]);
    // }
}

pub fn iter_seq_bin<'a>(
    path: &'a str,
    read_lens: &'a [u32],
    n_sites: &'a [Vec<usize>],
    offset: u64,
) -> impl Iterator<Item = Vec<u8>> + 'a {
    fn init_seqbits() -> Vec<Vec<u8>> {
        let bases = [b'A', b'C', b'G', b'T'];
        let mut seqbits = Vec::with_capacity(256);
        for i in 0..256 {
            seqbits.push(vec![
                bases[(i >> 6) & 3],
                bases[(i >> 4) & 3],
                bases[(i >> 2) & 3],
                bases[i & 3],
            ]);
        }
        seqbits
    }

    let seqbits = init_seqbits();
    let mut reader = BufReader::new(File::open(path).unwrap());
    reader
        .seek(std::io::SeekFrom::Start(offset))
        .expect("Failed to seek the ec bin file");

    let mut seq = Vec::new();
    let mut buffer = Vec::new();
    read_lens
        .iter()
        .map(|x| *x as usize)
        .enumerate()
        .map(move |(i, len)| {
            buffer.clear();
            buffer.resize(len / 4 + 1, 0);
            reader
                .read_exact(&mut buffer[..])
                .expect("Failed to parse the ec bin file");
            seq.clear();
            seq.extend(buffer.iter().flat_map(|x| &seqbits[*x as usize]).take(len));
            for p in n_sites[i].iter() {
                seq[*p] = b'N';
            }
            seq.to_owned()
        })
}

type Ovl = (usize, u32, u32, char, usize, u32, u32, u32, u32);
pub fn iter_ovl_bin(path: &str) -> impl Iterator<Item = Ovl> {
    let mut buffer = [0u8; 42];
    let mut length = 0;
    let mut reader = BufReader::new(File::open(path).unwrap());
    reader
        .read_exact(&mut buffer[..8])
        .expect("Failed to parse the ovl bin file");
    let mut read_count = i64::from_ne_bytes(buffer[..8].try_into().unwrap());
    std::iter::from_fn(move || {
        if read_count >= 0 {
            while length == 0 {
                if read_count == 0 {
                    return None;
                }
                reader
                    .read_exact(&mut buffer[..6])
                    .expect("Failed to parse the ovl bin file");
                length = i32::from_ne_bytes(buffer[2..6].try_into().unwrap());
                read_count -= 1;
            }
            length -= 1;
            reader
                .read_exact(&mut buffer)
                .expect("Failed to parse the ovl bin file");
            let (qns, qe, tn, ts, te, _, _, _, std, _, _) = (
                u64::from_ne_bytes(buffer[..8].try_into().unwrap()),
                u32::from_ne_bytes(buffer[8..12].try_into().unwrap()),
                u32::from_ne_bytes(buffer[12..16].try_into().unwrap()),
                u32::from_ne_bytes(buffer[16..20].try_into().unwrap()),
                u32::from_ne_bytes(buffer[20..24].try_into().unwrap()),
                buffer[24],
                buffer[25],
                u32::from_ne_bytes(buffer[26..30].try_into().unwrap()),
                u32::from_ne_bytes(buffer[30..34].try_into().unwrap()),
                u32::from_ne_bytes(buffer[34..38].try_into().unwrap()),
                u32::from_ne_bytes(buffer[38..].try_into().unwrap()),
            );
            let (qn, qs) = ((qns >> 32) as usize, (qns & 0xFFFFFFFF) as u32);
            // let ql = read_lens[qn as usize];
            // let tl = read_lens[tn as usize];
            // assert!(ql >= qe && tl >= te);

            let (mch, aln) = if qe - qs < te - ts {
                (qe - qs, te - ts)
            } else {
                (te - ts, qe - qs)
            };
            let std = if std == 1 { '-' } else { '+' };
            return Some((qn, qs, qe, std, tn as usize, ts, te, mch, aln));//qe - 1. te - 1
        }
        None
    })
}
