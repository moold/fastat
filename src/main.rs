use byte_unit::Byte;
use clap::{AppSettings, Arg, Command};
use crossbeam_channel::unbounded;
use crossbeam_utils::thread;
use rayon::prelude::*;
use std::{cmp::max, fmt};

#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;
#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

mod fofn;
mod io;
use fofn::open_path;
use io::Buffer;

pub const VERSION: &str = include_str!(concat!(env!("OUT_DIR"), "/VERSION"));
const BUF_COUNT: usize = 2;

#[derive(Default)]
struct Nx {
    count: [usize; 10],
    len: [u32; 10],
}

impl Nx {
    fn new() -> Self {
        Default::default()
    }

    fn get_width(&self) -> (usize, usize) {
        let mut w1 = self.count[8].to_string().len();
        let mut w2 = self.len[0].to_string().len();
        if w1 < 9 {
            w1 = 9;
        }
        if w2 < 11 {
            w2 = 11;
        }
        (w1, w2)
    }

    fn fill(mut self, lens: &[u32], total: usize) -> Self {
        let mut i = 0;
        let mut acc = 0;
        for len in lens.iter().rev() {
            acc += *len as usize;
            self.count[i] += 1;
            while acc as f64 > ((i + 1) * total) as f64 * 0.1 {
                self.len[i] = *len;
                i += 1;
                if i >= 10 {
                    break;
                }
                self.count[i] += self.count[i - 1];
            }
        }
        self
    }
}

impl fmt::Display for Nx {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (w1, w2) = self.get_width();
        writeln!(
            f,
            "{:<5} {:^w1$} {:^w2$}",
            "Types", "Count (#)", "Length (bp)",
        )?;
        for i in 0..9 {
            writeln!(
                f,
                "N{:<4} {:^w1$} {:^w2$}",
                (i + 1) * 10,
                self.count[i],
                self.len[i],
            )?;
        }
        Ok(())
    }
}

const BIN_LEN: u32 = 30;
const UNIT_BIN: usize = 200;
struct His {
    pw: usize,   // pos width
    cw: usize,   // count width
    unit: usize, // count of a '*'
    step: u32,
    start: u32,
    min: u32,
    max: u32,
    count: [usize; BIN_LEN as usize],
}

impl His {
    fn new(s: u32, e: u32, c: usize) -> Self {
        let step = max((e - s) / (BIN_LEN - 2), 1);
        Self {
            pw: e.to_string().len(),
            cw: c.to_string().len(),
            unit: max(c / UNIT_BIN, 5),
            step,
            start: s / step * step,
            min: u32::MIN,
            max: u32::MAX,
            count: [0; BIN_LEN as usize],
        }
    }

    fn fill(mut self, lens: &[u32]) -> Self {
        if !lens.is_empty() {
            let mut idx: usize = 0;
            let mut max_len = self.start;
            for len in lens {
                while idx + 1 < BIN_LEN as usize && *len >= max_len {
                    idx += 1;
                    max_len += self.step;
                }
                self.count[idx] += 1;
            }
            self.min = lens[0];
            self.max = *lens.last().unwrap();
            self.pw = self.max.to_string().len();
        }
        self
    }
}

impl fmt::Display for His {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "[length histogram ('*' =~ {} reads)]", self.unit)?;
        for (p, v) in self.count.into_iter().enumerate() {
            let p = p as u32;
            if v > 0 {
                writeln!(
                    f,
                    "{:>pw$} {:>pw$} {:>cw$} {:*>cv$}",
                    if p == 0 {
                        self.min
                    } else {
                        self.start + self.step * (p - 1)
                    },
                    if p == BIN_LEN - 1 {
                        self.max
                    } else {
                        self.start + self.step * p - 1
                    },
                    v,
                    "",
                    pw = self.pw,
                    cw = self.cw,
                    cv = v / self.unit
                )?;
            }
        }
        Ok(())
    }
}

fn out_stat(lens: &[u32], total: usize) {
    let total_count = lens.len();
    let (hist, nx) = thread::scope(|work| {
        let hist = work.spawn(move |_| {
            let dev = total_count / UNIT_BIN * 2;
            His::new(lens[dev], lens[total_count - max(1, dev)], total_count).fill(lens)
        });

        let nx = work.spawn(move |_| Nx::new().fill(lens, total));
        (
            hist.join().expect("Failed to generate histogram!"),
            nx.join().expect("Failed to generate Nx stats!"),
        )
    })
    .unwrap();

    println!("{}", hist);
    println!("\n\n[length stat]\n{}", nx);
    let (sw1, sw2) = nx.get_width();
    println!("{:<5} {:^sw1$} {:^sw2$}", "Min.", "-", lens[0],);
    println!(
        "{:<5} {:^sw1$} {:^sw2$}",
        "Max.",
        "-",
        lens[total_count - 1],
    );
    println!("{:<5} {:^sw1$} {:^sw2$}", "Ave.", "-", total / total_count,);
    println!("{:<5} {:^sw1$} {:^sw2$}", "Total", total_count, total,);
}

fn main() {
    let args = Command::new("ft")
        .version(VERSION)
        .about("simple statistics of FASTA/Q files")
        .arg_required_else_help(true)
        .global_setting(AppSettings::DeriveDisplayOrder)
        .arg(
            Arg::new("input")
                .required(true)
                .multiple_occurrences(true)
                .help("input file in [GZIP] FASTA/FASTQ/FOFN format."),
        )
        .arg(
            Arg::new("min_len")
                .short('m')
                .long("min_len")
                .value_name("int[G|M|K]")
                .default_value("0")
                .help("minimum sequence length, shorter sequences are ignored")
                .takes_value(true),
        )
        .get_matches();

    let min_len = Byte::from_str(args.value_of("min_len").unwrap())
        .unwrap()
        .get_bytes() as usize;

    // exit if any thread panics
    let orig_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |v| {
        orig_hook(v);
        std::process::exit(1);
    }));

    let (mut lens, total) = thread::scope(|work| {
        let (s1, r1) = unbounded();
        let (s2, r2) = unbounded();
        for _ in 0..BUF_COUNT {
            s1.send(Buffer::new()).unwrap();
        }

        // read thread
        work.spawn(move |_| {
            for infile in args.values_of("input").expect("Missing input file!") {
                for mut reader in open_path(&infile) {
                    loop {
                        let mut buf = r1.recv().unwrap();
                        match buf.fill(&mut reader) {
                            Ok(0) => {
                                let is_empty = buf.is_empty();
                                s2.send(Some(buf)).unwrap(); //send an empty buf to indicate this file reaches EOF
                                if is_empty {
                                    break;
                                }
                            }
                            Ok(_n) => s2.send(Some(buf)).unwrap(),
                            Err(e) => panic!("Failed to read file: {:?}, error: {:?}", infile, e),
                        }
                    }
                }
            }
            s2.send(None).unwrap();
            while r1.len() != BUF_COUNT {} //wait the stat thread to finish
        });

        // statistics thread
        let stat = work.spawn(move |_| {
            let mut skip_lines = 0;
            let mut skip_bases = 0;
            let mut is_new_record = true;

            let mut len = 0;
            let mut lens = Vec::with_capacity(1024000);
            let mut total: usize = 0;
            while let Ok(Some(mut buf)) = r2.recv() {
                if buf.is_empty() {
                    // a file has reached EOF
                    if skip_bases != 0 || skip_lines != 0 {
                        panic!("truncate file");
                    } else if len > min_len {
                        // save the last fasta record
                        lens.push(len as u32);
                        total += len;
                    }
                    len = 0;
                    is_new_record = true;
                    s1.send(buf).unwrap();
                    continue;
                }

                if skip_lines > 0 {
                    let skip_line = buf.skip_lines(skip_lines);
                    skip_lines -= skip_line;
                    if skip_lines > 0 {
                        s1.send(buf).unwrap();
                        continue;
                    }
                }

                if skip_bases > 0 {
                    let skip_base = buf.skip_bases(skip_bases);
                    skip_bases -= skip_base;
                    if skip_bases > 0 {
                        s1.send(buf).unwrap();
                        continue;
                    }
                }

                'outer: while let Some(c) = buf.next_byte(true) {
                    if !is_new_record {
                        if c == b'>' || c == b'@' {
                            if len > min_len {
                                // save the previous fasta record
                                lens.push(len as u32);
                                total += len;
                            }
                            len = 0;
                            is_new_record = true;
                            continue;
                        } else if c == b'+' {
                            if len > min_len {
                                // save the previous fasta record
                                lens.push(len as u32);
                                total += len;
                            }
                            let l = buf.skip_bases(len + 1); // skip sep and qual
                            is_new_record = true;
                            if l != len + 1 {
                                skip_bases = len + 1 - l;
                                len = 0;
                                break;
                            } else {
                                len = 0;
                                continue;
                            }
                        }
                    } else if c == b'>' || c == b'@' {
                        is_new_record = false;
                        let skip_line = buf.skip_lines(1); //skip head
                        if skip_line != 1 {
                            skip_lines = 1;
                            break;
                        }
                    } else if is_new_record && c != b'>' && c != b'@' {
                        panic!("Not a correct fasta/fastq file");
                    }

                    while let Some((p, _a)) = buf.next_line_len() {
                        // iter seq
                        len += p;
                        if let Some(c) = buf.next_byte(false) {
                            if c == b'>' {
                                // fasta
                                break;
                            } else if c == b'+' {
                                // fastq
                                let l = buf.skip_bases(len + 1); // skip sep and qual
                                if l != len + 1 {
                                    skip_bases = len + 1 - l;
                                }
                                break;
                            }
                        } else {
                            //need continue to read from the next buf
                            break 'outer;
                        }
                    }
                    if len > 0 {
                        if len > min_len {
                            lens.push(len as u32);
                            total += len;
                        }
                        len = 0;
                        is_new_record = true;
                    }
                }
                s1.send(buf).unwrap();
            }
            (lens, total)
        });
        stat.join().expect("Failed to read from input file!")
    })
    .unwrap();

    if !lens.is_empty() {
        lens.par_sort_unstable();
        out_stat(&lens, total);
    }
}
