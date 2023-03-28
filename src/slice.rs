use core::mem::size_of;

use crate::{
    buffer::Buffer,
    formula::{BareFormula, Formula},
    iter::default_iter_fast_sizes,
    serialize::{write_slice, Serialize, Sizes},
    size::FixedUsize,
};

impl<F> Formula for [F]
where
    F: Formula,
{
    const MAX_STACK_SIZE: Option<usize> = match F::MAX_STACK_SIZE {
        Some(0) => Some(size_of::<FixedUsize>()),
        Some(_) => None,
        None => None,
    };
    const EXACT_SIZE: bool = matches!(F::MAX_STACK_SIZE, Some(0));
    const HEAPLESS: bool = F::HEAPLESS;
}

impl<F> BareFormula for [F] where F: Formula {}

impl<'ser, F, T> Serialize<[F]> for &'ser [T]
where
    F: Formula,
    &'ser T: Serialize<F>,
{
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        Self: Sized,
        B: Buffer,
    {
        write_slice(self.iter(), sizes, buffer)
    }

    fn size_hint(&self) -> Option<Sizes> {
        Some(Sizes::with_stack(default_iter_fast_sizes::<F, _>(
            &self.iter(),
        )?))
    }
}
