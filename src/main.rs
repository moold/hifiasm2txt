use gzp::{
    deflate::Gzip,
    par::compress::{ParCompress, ParCompressBuilder},
    ZWriter,
};
use std::{fs::File, io::Write};

mod hifiasm;
mod option;
mod resource;
use hifiasm::{iter_ovl_bin, iter_seq_bin, read_seq_info};
use option::Option as Opt;

fn main() {
    let opt = Opt::from_args();
    let chunksize = 1 << 20;
    let mut buffer = Vec::with_capacity(chunksize);

    let (offset, read_lens, read_names, n_sites) = read_seq_info(&opt.in_ec_bin);
    if let Some(out_file) = opt.out_ec_bin {
        let file = File::create(out_file).expect("Unable to create file");
        let mut writer: ParCompress<Gzip> = ParCompressBuilder::new().from_writer(file);
        for (idx, seq) in iter_seq_bin(&opt.in_ec_bin, &read_lens, &n_sites, offset).enumerate() {
            buffer.push(b'>');
            buffer.extend(&read_names[idx]);
            buffer.push(b'\n');
            buffer.extend(seq);
            buffer.push(b'\n');
            if buffer.len() >= chunksize {
                writer.write_all(&buffer).expect("Unable to write to file!");
                buffer.clear();
            }
        }

        if !buffer.is_empty() {
            writer.write_all(&buffer).expect("Unable to write to file!");
            buffer.clear();
        }
        writer.finish().expect("Unable to write to file!");
    }

    let mut ovl_bins = Vec::with_capacity(2);
    if let Some(out_so_bin) = opt.out_so_bin {
        ovl_bins.push((opt.in_so_bin, out_so_bin));
    }
    if let Some(out_re_bin) = opt.out_re_bin {
        ovl_bins.push((opt.in_re_bin, out_re_bin));
    }

    for (in_file, out_file) in ovl_bins {
        let file = File::create(out_file).expect("Unable to create file");
        let mut writer: ParCompress<Gzip> = ParCompressBuilder::new().from_writer(file);
        for (qn, qs, qe, std, tn, ts, te, mch, aln) in iter_ovl_bin(&in_file) {
            let (ql, tl) = (read_lens[qn], read_lens[tn]);
            let (qn, tn) = (&read_names[qn], &read_names[tn]);
            buffer.extend(qn);
            write!(&mut buffer, "\t{ql}\t{qs}\t{qe}\t{std}\t").unwrap();
            buffer.extend(tn);
            writeln!(&mut buffer, "\t{tl}\t{ts}\t{te}\t{mch}\t{aln}").unwrap();
            if buffer.len() >= chunksize {
                writer.write_all(&buffer).expect("Unable to write to file!");
                buffer.clear();
            }
        }
        if !buffer.is_empty() {
            writer.write_all(&buffer).expect("Unable to write to file!");
            buffer.clear();
        }
        writer.finish().expect("Unable to write to file!");
    }

    eprintln!("{}", resource::resource_str());
}
