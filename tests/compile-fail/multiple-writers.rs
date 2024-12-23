use sharded_vec_writer::VecWriter;

fn main() {
    let mut v: Vec<u32> = Vec::with_capacity(2);
    let mut writer1 = VecWriter::new(&mut v);
    let mut writer2 = VecWriter::new(&mut v);
    let mut shard1 = writer1.take_shard(2);
    let mut shard2 = writer2.take_shard(2);
    shard1.push(1);
    shard2.push(1);
}
