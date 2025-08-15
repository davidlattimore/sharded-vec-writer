fn main() {
    use sharded_vec_writer::VecWriter;

    let shard_sizes = [4, 3, 7];
    let total_size = shard_sizes.iter().sum();

    // Create the vec with sufficient capacity for whatever we'd like to put in it.
    let mut data = Vec::with_capacity(total_size);

    // Create a writer - this mutably borrows the vec.
    let mut writer = VecWriter::new(&mut data);

    // Split the writer into some number of shards up to the capacity of the Vec.
    let mut shards = writer.take_shards(shard_sizes.into_iter());

    std::thread::scope(|scope| {
        // Write to the shards. This can be done from separate threads.
        for shard in &mut shards {
            scope.spawn(|| {
                for i in 0..shard.len() {
                    shard.push(i);
                }
            });
        }
    });

    // This resizes the Vec to have length `total_size`. The shards need to all have been fully
    // written, otherwise this will panic. For a non-panicking version, you can use
    // `try_return_shards`.
    writer.return_shards(shards);

    assert_eq!(data, &[0, 1, 2, 3, 0, 1, 2, 0, 1, 2, 3, 4, 5, 6]);
}
