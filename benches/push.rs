use criterion::{
    criterion_group, criterion_main, AxisScale, BenchmarkId, Criterion, PlotConfiguration,
};
use vector::myvec::MyVec;

fn push_without_realloc_with_initial_alloc_stdvec(vec: &mut Vec<usize>, count: usize) {
    for i in (0..count).into_iter() {
        vec.push(i)
    }
}
fn push_without_realloc_with_initial_alloc_myvec(vec: &mut MyVec<usize>, count: usize) {
    for i in (0..count).into_iter() {
        vec.push(i)
    }
}

fn bench_push_with_realloc(c: &mut Criterion) {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);

    let mut group = c.benchmark_group("PushUsize");
    group.plot_config(plot_config);

    for count in [0, 1, 4, 8, 16, 1024, 4096].iter() {
        group.bench_with_input(BenchmarkId::new("stdvec", count), count, |b, count| {
            b.iter(|| push_without_realloc_with_initial_alloc_stdvec(&mut vec![], *count))
        });
        group.bench_with_input(BenchmarkId::new("MyVec", count), count, |b, count| {
            b.iter(|| push_without_realloc_with_initial_alloc_myvec(&mut MyVec::new(), *count))
        });
    }
    group.finish();
}

fn bench_push_without_realloc(c: &mut Criterion) {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);

    let mut group = c.benchmark_group("ReusedPushUsize");
    group.plot_config(plot_config);

    for count in [0, 1, 4, 8, 16, 1024, 4096].iter() {
        group.bench_with_input(BenchmarkId::new("stdvec", count), count, |b, count| {
            let mut vec = Vec::with_capacity(*count);
            b.iter(|| {
                push_without_realloc_with_initial_alloc_stdvec(&mut vec, *count);
                vec.truncate(0);
            })
        });
        group.bench_with_input(BenchmarkId::new("MyVec", count), count, |b, count| {
            let mut vec = MyVec::with_capacity(*count);
            b.iter(|| {
                push_without_realloc_with_initial_alloc_myvec(&mut vec, *count);
                vec.truncate(0);
            })
        });
    }
    group.finish();
}

fn bench_push_without_realloc_with_initial_alloc(c: &mut Criterion) {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);

    let mut group = c.benchmark_group("PreallocatedPushUsize");
    group.plot_config(plot_config);

    for count in [0, 1, 4, 8, 16, 1024, 4096].iter() {
        group.bench_with_input(BenchmarkId::new("stdvec", count), count, |b, count| {
            b.iter(|| {
                push_without_realloc_with_initial_alloc_stdvec(
                    &mut Vec::with_capacity(*count),
                    *count,
                )
            })
        });
        group.bench_with_input(BenchmarkId::new("MyVec", count), count, |b, count| {
            b.iter(|| {
                push_without_realloc_with_initial_alloc_myvec(
                    &mut MyVec::with_capacity(*count),
                    *count,
                )
            })
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_push_without_realloc,
    bench_push_without_realloc_with_initial_alloc,
    bench_push_with_realloc
);
criterion_main!(benches);
