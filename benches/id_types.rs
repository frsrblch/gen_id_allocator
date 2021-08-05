use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::hash::{Hash, Hasher};
use std::num::{NonZeroU32, NonZeroU64};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct PackedId(NonZeroU64);

impl PackedId {
    pub fn new(index: u32) -> Self {
        let index = (index as u64) << 32 | 1u64;
        let index = NonZeroU64::new(index).unwrap();
        Self(index)
    }

    pub fn index(self) -> usize {
        (self.0.get() >> 32) as usize
    }

    pub fn gen(self) -> NonZeroU32 {
        let gen = (self.0.get() & 0x0000_ffff) as u32;
        NonZeroU32::new(gen).unwrap()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct SplitId {
    index: u32,
    gen: NonZeroU32,
}

impl SplitId {
    pub fn new(index: u32) -> Self {
        Self {
            index: index,
            gen: NonZeroU32::new(1).unwrap(),
        }
    }

    pub fn index(self) -> usize {
        self.index as usize
    }

    pub fn gen(self) -> NonZeroU32 {
        self.gen
    }
}

criterion_main! {
    id_types,
}

criterion_group! {
    id_types,
    packed_id,
    split_id,
}

const N: u32 = 1024;

fn packed_id(c: &mut Criterion) {
    let id = PackedId::new(2);
    assert_eq!(2, id.index());

    let ids: Vec<_> = (0..N).into_iter().map(PackedId::new).collect();

    c.bench_function("packed_id_hash", |b| {
        b.iter(|| {
            // let mut hasher = fnv::FnvHasher::default();
            let mut hasher = ahash::AHasher::default();

            ids.iter().for_each(|id| id.hash(&mut hasher));

            black_box(hasher.finish());
        })
    })
    .bench_function("packed_id_eq", |b| {
        b.iter(|| {
            black_box(ids.iter().zip(&ids).all(|(a, b)| a.eq(b)));
        })
    })
    .bench_function("packed_id_index", |b| {
        b.iter(|| {
            black_box(ids.iter().copied().map(PackedId::index).sum::<usize>());
        })
    })
    .bench_function("packed_id_gen", |b| {
        b.iter(|| {
            black_box(
                ids.iter()
                    .copied()
                    .map(PackedId::gen)
                    .map(NonZeroU32::get)
                    .sum::<u32>(),
            );
        })
    });
}

fn split_id(c: &mut Criterion) {
    let ids: Vec<_> = (0..N).into_iter().map(SplitId::new).collect();

    c.bench_function("split_id_hash", |b| {
        b.iter(|| {
            // let mut hasher = fnv::FnvHasher::default();
            let mut hasher = ahash::AHasher::default();

            ids.iter().for_each(|id| {
                id.hash(&mut hasher);
            });

            black_box(hasher.finish());
        })
    })
    .bench_function("split_id_eq", |b| {
        b.iter(|| {
            black_box(ids.iter().zip(&ids).all(|(a, b)| a.eq(b)));
        })
    })
    .bench_function("split_id_index", |b| {
        b.iter(|| {
            black_box(ids.iter().copied().map(SplitId::index).sum::<usize>());
        })
    })
    .bench_function("split_id_gen", |b| {
        b.iter(|| {
            black_box(
                ids.iter()
                    .copied()
                    .map(SplitId::gen)
                    .map(NonZeroU32::get)
                    .sum::<u32>(),
            );
        })
    });
}
