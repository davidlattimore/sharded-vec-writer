use sharded_vec_writer::VecWriter;

fn main() {
    let mut v: Vec<u32> = Vec::with_capacity(2);
    let mut writer = VecWriter::new(&mut v);
    let mut shard = writer.take_shard(2);
    shard.push(1);
    shard.push(2);
    let m1 = shard.init_mut();
    let m2 = shard.init_mut();
    m1[0] = 2;
    m2[0] = 5;
}
