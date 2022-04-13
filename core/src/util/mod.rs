pub use bitflags::Bitflags;

mod bitflags {
    pub trait Bitflags: Copy + Into<u8> {
        fn get(self, f: &u8) -> bool {
            *f & self.into() != 0
        }

        fn set(self, f: &mut u8, enable: bool) {
            *f ^= (*f & self.into()) ^ (!(enable as u8).wrapping_sub(1) & self.into())
        }
    }
}
