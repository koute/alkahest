use {
    crate::{Pack, Schema, SchemaUnpack, Unpacked},
    core::{
        convert::TryFrom,
        marker::PhantomData,
        mem::{align_of, size_of},
    },
};

/// Type used to represent sizes and offsets in alkahest packages.
/// This places limitation on sequence sizes which practically is never hit.
/// `usize` itself is not portable and cannot be written into alkahest package.
type FixedUsize = u32;

#[derive(Clone, Copy)]
pub struct SeqUnpacked<'a, T> {
    offset: usize,
    len: usize,
    bytes: &'a [u8],
    marker: PhantomData<[T]>,
}

pub struct Seq<T> {
    marker: PhantomData<[T]>,
}

impl<'a, T> SchemaUnpack<'a> for Seq<T>
where
    T: Schema,
{
    type Unpacked = SeqUnpacked<'a, T>;
}

impl<T> Schema for Seq<T>
where
    T: Schema,
{
    type Packed = [FixedUsize; 2];

    fn align() -> usize {
        1 + ((align_of::<[FixedUsize; 2]>() - 1) | (<T as Schema>::align() - 1))
    }

    fn unpack<'a>(packed: [FixedUsize; 2], bytes: &'a [u8]) -> SeqUnpacked<'a, T> {
        SeqUnpacked {
            len: usize::try_from(packed[0]).expect("Sequence is too large"),
            offset: usize::try_from(packed[1]).expect("Package is too large"),
            bytes,
            marker: PhantomData,
        }
    }
}

#[cfg(target_endian = "little")]
impl<'a, T> SeqUnpacked<'a, T> {
    /// View sequence of `Pod` values are a slice.
    pub fn as_slice(&self) -> &[T]
    where
        T: bytemuck::Pod + Schema<Packed = T>,
    {
        bytemuck::cast_slice(&self.bytes[self.offset..][..size_of::<T>() * self.len])
    }
}

impl<'a, T> Iterator for SeqUnpacked<'a, T>
where
    T: Schema,
{
    type Item = Unpacked<'a, T>;

    fn next(&mut self) -> Option<Unpacked<'a, T>> {
        if self.len == 0 {
            None
        } else {
            let item = *bytemuck::from_bytes(&self.bytes[self.offset..][..size_of::<T::Packed>()]);
            self.offset += size_of::<T::Packed>();
            self.len -= 1;
            Some(T::unpack(item, self.bytes))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T> ExactSizeIterator for SeqUnpacked<'a, T>
where
    T: Schema,
{
    fn len(&self) -> usize {
        self.len
    }
}

impl<I, T> Pack<Seq<T>> for I
where
    T: Schema,
    I: IntoIterator,
    I::IntoIter: ExactSizeIterator,
    I::Item: Pack<T>,
{
    fn pack(self, offset: usize, bytes: &mut [u8]) -> ([FixedUsize; 2], usize) {
        let iter = self.into_iter();
        let len = iter.len();

        let len32 = u32::try_from(len).expect("Sequence is too large");
        let offset32 = u32::try_from(offset).expect("Sequence is too large");

        let packed_size = size_of::<T::Packed>();

        let mut used = packed_size * len;

        let mut off = 0;
        for item in iter {
            let (item_packed, item_used) = item.pack(offset + used, &mut bytes[used..]);
            bytes[off..][..size_of::<T::Packed>()]
                .copy_from_slice(bytemuck::bytes_of(&item_packed));
            used += item_used;
            off += size_of::<T::Packed>();
        }

        ([len32, offset32], used)
    }
}