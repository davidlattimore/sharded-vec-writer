fn main() {
    use sharded_vec_writer::VecWriter;

    // Create the vec with sufficient capacity for whatever we'd like to put in it.
    let mut v = Vec::with_capacity(20);

    // Create a writer - this mutably borrows the vec.
    let mut writer: VecWriter<u32> = VecWriter::new(&mut v);

    // Create however many shards we'd like, up to the capacity of the vec.
    let mut shard1 = writer.take_shard(8);
    let mut shard2 = writer.take_shard(2);
    let mut shard3 = writer.take_shard(10);

    // Write to the shards, possibly from multiple threads. Scoped threads help here.
    std::thread::scope(|scope| {
        scope.spawn(|| {
            for i in 0..8 {
                shard1.push(i);
            }
        });
        scope.spawn(|| {
            for i in 8..10 {
                shard2.push(i);
            }
        });
    });
    for i in 10..20 {
        shard3.push(i);
    }

    // Return the shards to the writer. Shards must be fully initialised and must be returned in
    // order.
    writer.return_shard(shard1);
    writer.return_shard(shard2);
    writer.return_shard(shard3);

    assert_eq!(v.len(), 20);
    assert_eq!(v.capacity(), 20);
    assert_eq!(v, (0..20).collect::<Vec<_>>());
}
