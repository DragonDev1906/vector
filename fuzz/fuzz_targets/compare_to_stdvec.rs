#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use vector::myvec::MyVec;

#[derive(Arbitrary, Debug)]
enum VecMethod {
    Push(u8),
    GetCapacity,
    Reserve(u16),
    ReserveExact(u16),
    ShrinkTo(u16),
    ShrinkToFit,
    Truncate(u16),
}

#[derive(Arbitrary, Debug)]
enum VecSetup {
    Empty,
    // Technically we could allocate more than this, but we'd run into memory issues if we do.
    Capacity(u16),
}

#[derive(Arbitrary, Debug)]
struct Target {
    setup: VecSetup,
    methods: Vec<VecMethod>,
}

fuzz_target!(|target: Target| {
    let (mut std_vec, mut my_vec) = match target.setup {
        VecSetup::Empty => (Vec::<u8>::new(), MyVec::<u8>::new()),
        VecSetup::Capacity(cap) => (
            Vec::with_capacity(cap.into()),
            MyVec::with_capacity(cap.into()),
        ),
    };

    for method in target.methods {
        match method {
            VecMethod::Push(value) => {
                std_vec.push(value);
                my_vec.push(value);
                assert!(std_vec.iter().eq(my_vec.iter()))
            }
            VecMethod::GetCapacity => {
                assert_eq!(std_vec.capacity(), my_vec.capacity())
            }
            VecMethod::Reserve(additional) => {
                std_vec.reserve(additional.into());
                my_vec.reserve(additional.into());
                assert_eq!(std_vec.len(), my_vec.len());
                assert_eq!(std_vec.capacity(), my_vec.capacity());
            }
            VecMethod::ReserveExact(additional) => {
                std_vec.reserve_exact(additional.into());
                my_vec.reserve_exact(additional.into());
                assert_eq!(std_vec.len(), my_vec.len());
                assert_eq!(std_vec.capacity(), my_vec.capacity());
            }
            VecMethod::ShrinkTo(cap) => {
                std_vec.shrink_to(cap.into());
                my_vec.shrink_to(cap.into());
                assert_eq!(std_vec.len(), my_vec.len());
                assert_eq!(std_vec.capacity(), my_vec.capacity());
            }
            VecMethod::ShrinkToFit => {
                std_vec.shrink_to_fit();
                my_vec.shrink_to_fit();
                assert_eq!(std_vec.len(), my_vec.len());
                assert_eq!(std_vec.capacity(), my_vec.capacity());
            }
            VecMethod::Truncate(len) => {
                std_vec.truncate(len.into());
                my_vec.truncate(len.into());
                assert_eq!(std_vec.len(), my_vec.len());
                assert_eq!(std_vec.capacity(), my_vec.capacity());
            }
        }
    }
});
