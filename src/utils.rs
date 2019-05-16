pub mod io {
    use std::io::{Read, Result, Write};

    pub struct ReadWithProgress<'a, T: Read + 'a, F: FnMut(usize, usize) -> () + 'a> {
        read: &'a mut T,
        size: usize,
        progress: Option<F>
    }

    impl<'a, T: Read + 'a, F: FnMut(usize, usize) -> () + 'a> ReadWithProgress<'a, T, F> {
        pub fn new(read: &'a mut T, size: usize, progress: Option<F>) -> Self {
            ReadWithProgress { read: read, size: size, progress: progress }
        }
    }

    impl<'a, T: Read + 'a, F: FnMut(usize, usize) -> () + 'a> Read for ReadWithProgress<'a, T, F> {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
            let result = self.read.read(buf);
            if self.progress.is_some() && result.is_ok() {
                let delta = *result.as_ref().unwrap();
                let  progress = self.progress.as_mut().unwrap();
                progress(self.size, delta);
            }
            result
        }
    }

    pub struct WriteWithProgress<'a, T: Write + 'a, F: FnMut(usize, usize) -> () + 'a> {
        write: &'a mut T,
        size: usize,
        progress: Option<F>
    }

    impl<'a, T: Write + 'a, F: FnMut(usize, usize) -> () + 'a> WriteWithProgress<'a, T, F> {
        pub fn new(write: &'a mut T, size: usize, progress: Option<F>) -> Self {
            WriteWithProgress { write: write, size: size, progress: progress }
        }
    }

    impl<'a, T: Write + 'a, F: FnMut(usize, usize) -> () + 'a> Write for WriteWithProgress<'a, T, F> {
        fn write(&mut self, buf: &[u8]) -> Result<usize> {
            let result = self.write.write(buf);
            if self.progress.is_some() && result.is_ok() {
                let delta = *result.as_ref().unwrap();
                let  progress = self.progress.as_mut().unwrap();
                progress(self.size, delta);
            }
            result
        }

        fn flush(&mut self) -> Result<()> {
            self.write.flush()
        }
    }

 }
