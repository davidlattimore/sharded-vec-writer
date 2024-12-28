use sharded_vec_writer::VecWriter;

fn main() {
    let mut v: Vec<u32> = Vec::with_capacity(2);
    let mut writer = VecWriter::new(&mut v);
    let mut shard = writer.take_shard(2);
    let v1 = shard.push(1);
    writer.return_shard(shard);
    println!("{v:?}");
    *v1 = 1;
    println!("{v:?}");
}
