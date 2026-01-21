//! Benchmarks for mesh generation performance.

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use voxel_forge::world::{Block, Chunk, ChunkPos, MeshGenerator};

/// Fill a chunk with terrain for benchmarking.
fn create_terrain_chunk(cx: i32, cz: i32) -> Chunk {
    use voxel_forge::world::SECTION_SIZE;

    let mut chunk = Chunk::new(ChunkPos::new(cx, cz));

    for x in 0..SECTION_SIZE {
        for z in 0..SECTION_SIZE {
            let wx = cx * SECTION_SIZE as i32 + x as i32;
            let wz = cz * SECTION_SIZE as i32 + z as i32;

            // Simple height variation
            let height = 64
                + ((wx as f32 * 0.1).sin() * 3.0) as usize
                + ((wz as f32 * 0.15).cos() * 2.0) as usize;

            chunk.set_block(x, 0, z, Block::Bedrock);

            for y in 1..height.saturating_sub(4) {
                chunk.set_block(x, y, z, Block::Stone);
            }

            for y in height.saturating_sub(4)..height {
                chunk.set_block(x, y, z, Block::Dirt);
            }

            chunk.set_block(x, height, z, Block::Grass);
        }
    }

    chunk
}

fn mesh_generation_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("mesh_generation");

    // Single chunk benchmark
    group.bench_function("single_chunk_terrain", |b| {
        let chunk = create_terrain_chunk(0, 0);
        b.iter(|| {
            let generator = MeshGenerator::new(black_box(chunk.clone()));
            black_box(generator.generate())
        });
    });

    // Solid 16x16x16 section (worst case for greedy - nothing to merge)
    group.bench_function("solid_section_16x16x16", |b| {
        let mut chunk = Chunk::new(ChunkPos::new(0, 0));
        for x in 0..16 {
            for y in 0..16 {
                for z in 0..16 {
                    // Alternate block types to prevent merging
                    let block = if (x + y + z) % 2 == 0 {
                        Block::Stone
                    } else {
                        Block::Dirt
                    };
                    chunk.set_block(x, y, z, block);
                }
            }
        }
        b.iter(|| {
            let generator = MeshGenerator::new(black_box(chunk.clone()));
            black_box(generator.generate())
        });
    });

    // Uniform layer (best case for greedy - maximum merging)
    group.bench_function("uniform_layer_16x16", |b| {
        let mut chunk = Chunk::new(ChunkPos::new(0, 0));
        for x in 0..16 {
            for z in 0..16 {
                chunk.set_block(x, 64, z, Block::Stone);
            }
        }
        b.iter(|| {
            let generator = MeshGenerator::new(black_box(chunk.clone()));
            black_box(generator.generate())
        });
    });

    group.finish();
}

fn parallel_meshing_benchmark(c: &mut Criterion) {
    use rayon::prelude::*;

    let mut group = c.benchmark_group("parallel_meshing");

    for chunk_count in [9, 25, 49] {
        let side = (chunk_count as f32).sqrt() as i32;
        let half = side / 2;

        group.bench_with_input(
            BenchmarkId::new("chunks", chunk_count),
            &chunk_count,
            |b, _| {
                let coords: Vec<(i32, i32)> = (-half..=half)
                    .flat_map(|cx| (-half..=half).map(move |cz| (cx, cz)))
                    .take(chunk_count)
                    .collect();

                b.iter(|| {
                    let meshes: Vec<_> = coords
                        .par_iter()
                        .map(|&(cx, cz)| {
                            let chunk = create_terrain_chunk(cx, cz);
                            let generator = MeshGenerator::new(chunk);
                            generator.generate()
                        })
                        .collect();
                    black_box(meshes)
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    mesh_generation_benchmark,
    parallel_meshing_benchmark
);
criterion_main!(benches);
