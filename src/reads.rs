use seq_io::fasta;
use seq_io::parallel::read_process_fasta_records;
pub use seq_io::BaseRecord;
use std::fs::File;
use std::path::Path;
use std::slice::Iter;

pub struct Fasta {
    reader: fasta::Reader<File>,
}

impl Fasta {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Self {
        Self {
            reader: fasta::Reader::from_path(path).expect("Failed to open file"),
        }
    }
}

pub trait ReadProcess: Sized {
    type Rec<'a>: BaseRecord;

    fn process_rec<F: FnMut(Self::Rec<'_>)>(self, f: F);

    fn process_rec_par_result<
        R: Default + Send,
        F: Send + Sync + Fn(Self::Rec<'_>, &mut R),
        G: FnMut(Self::Rec<'_>, &mut R),
    >(
        self,
        threads: u32,
        queue_len: usize,
        f: F,
        handle_result: G,
    );

    #[inline]
    fn process<F: FnMut(Iter<u8>)>(self, mut f: F) {
        self.process_rec(|record| f(record.seq().iter()));
    }

    #[inline]
    fn process_rec_par<F: Send + Sync + Fn(Self::Rec<'_>)>(
        self,
        threads: u32,
        queue_len: usize,
        f: F,
    ) {
        self.process_rec_par_result(
            threads,
            queue_len,
            |record, _: &mut Option<()>| f(record),
            |_, _| (),
        )
    }

    #[inline]
    fn process_par<F: Send + Sync + Fn(Iter<u8>)>(self, threads: u32, queue_len: usize, f: F) {
        self.process_rec_par(threads, queue_len, |record| f(record.seq().iter()));
    }

    #[inline]
    fn process_par_result<
        R: Default + Send,
        F: Send + Sync + Fn(Iter<u8>, &mut R),
        G: FnMut(&mut R),
    >(
        self,
        threads: u32,
        queue_len: usize,
        f: F,
        mut handle_result: G,
    ) {
        self.process_rec_par_result(
            threads,
            queue_len,
            |record, result| f(record.seq().iter(), result),
            |_, result| handle_result(result),
        );
    }
}

impl ReadProcess for Fasta {
    type Rec<'a> = fasta::RefRecord<'a>;

    fn process_rec<F: FnMut(Self::Rec<'_>)>(mut self, mut f: F) {
        while let Some(result) = self.reader.next() {
            let record = result.expect("Error reading record");
            f(record);
        }
    }

    fn process_rec_par_result<
        R: Default + Send,
        F: Send + Sync + Fn(Self::Rec<'_>, &mut R),
        G: FnMut(Self::Rec<'_>, &mut R),
    >(
        self,
        threads: u32,
        queue_len: usize,
        f: F,
        mut handle_result: G,
    ) {
        read_process_fasta_records(
            self.reader,
            threads,
            queue_len,
            |record: Self::Rec<'_>, result: &mut R| {
                f(record, result);
            },
            |record, result| {
                handle_result(record, result);
                None::<()>
            },
        )
        .unwrap();
    }
}
