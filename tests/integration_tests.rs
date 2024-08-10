use sharded_vec_writer::InitError;
use sharded_vec_writer::InsufficientCapacity;
use sharded_vec_writer::VecWriter;
use std::rc::Rc;

#[test]
fn basic_usage() {
    let mut v = Vec::with_capacity(20);
    let mut writer: VecWriter<u32> = VecWriter::new(&mut v);
    let mut shard1 = writer.take_shard(8);
    let mut shard2 = writer.take_shard(2);
    let mut shard3 = writer.take_shard(10);
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
    writer.return_shard(shard1);
    writer.return_shard(shard2);
    writer.return_shard(shard3);

    assert_eq!(v.len(), 20);
    assert_eq!(v.capacity(), 20);
    assert_eq!(v, (0..20).collect::<Vec<_>>());
}

#[test]
fn empty() {
    let mut v = Vec::with_capacity(0);
    let mut writer: VecWriter<u32> = VecWriter::new(&mut v);
    assert!(writer.try_take_shard(8).is_none());
}

#[test]
fn partial_return() {
    let mut v = Vec::with_capacity(10);
    let mut writer: VecWriter<u32> = VecWriter::new(&mut v);
    let mut shard1 = writer.take_shard(8);
    for i in 0..8 {
        shard1.push(i);
    }
    writer.return_shard(shard1);
    assert_eq!(v.len(), 8);
}

#[test]
fn return_to_wrong_vec() {
    let mut v1 = Vec::with_capacity(10);
    let mut writer1: VecWriter<u32> = VecWriter::new(&mut v1);
    let mut shard1 = writer1.take_shard(8);

    let mut v2 = Vec::with_capacity(10);
    let mut writer2: VecWriter<u32> = VecWriter::new(&mut v2);
    let mut shard2 = writer2.take_shard(8);

    for i in 0..8 {
        shard1.push(i);
    }
    for i in 0..8 {
        shard2.push(i);
    }

    assert_eq!(
        writer1.try_return_shard(shard2).unwrap_err(),
        InitError::WrongVec
    );
    writer1.return_shard(shard1);
}

#[test]
fn missing_shard() {
    let mut v = Vec::with_capacity(10);
    let mut writer: VecWriter<u32> = VecWriter::new(&mut v);
    let mut shard1 = writer.take_shard(4);
    let mut shard2 = writer.take_shard(4);

    for i in 0..4 {
        shard1.push(i);
    }
    for i in 0..4 {
        shard2.push(i);
    }

    assert_eq!(
        writer.try_return_shard(shard2).unwrap_err(),
        InitError::OutOfOrder
    );
    writer.return_shard(shard1);
}

#[test]
fn not_fully_initialised() {
    let mut v = Vec::with_capacity(10);
    let mut writer: VecWriter<u32> = VecWriter::new(&mut v);
    let mut shard1 = writer.take_shard(4);

    for i in 0..2 {
        shard1.push(i);
    }

    assert_eq!(
        writer.try_return_shard(shard1).unwrap_err(),
        InitError::UninitElements
    );
}

#[test]
fn push_too_much() {
    let mut v = Vec::with_capacity(10);
    let mut writer: VecWriter<u32> = VecWriter::new(&mut v);
    let mut shard1 = writer.take_shard(2);

    for i in 0..2 {
        shard1.push(i);
    }
    assert_eq!(shard1.try_push(4).unwrap_err(), InsufficientCapacity);
    writer.return_shard(shard1);
}

#[test]
fn non_copy_type() {
    let mut v = Vec::with_capacity(2);
    let mut writer: VecWriter<Vec<u32>> = VecWriter::new(&mut v);
    let mut shard1 = writer.take_shard(2);

    shard1.push(vec![1, 2, 3]);
    shard1.push(vec![4, 5, 6]);

    writer.return_shard(shard1);

    assert_eq!(v[0], vec![1, 2, 3]);
    assert_eq!(v[1], vec![4, 5, 6]);
}

#[test]
fn drop_without_returning() {
    let mut v = Vec::with_capacity(2);
    let mut writer: VecWriter<Rc<()>> = VecWriter::new(&mut v);
    let mut shard1 = writer.take_shard(2);

    let r = Rc::new(());
    shard1.push(Rc::clone(&r));

    assert_eq!(Rc::strong_count(&r), 2);
    drop(shard1);
    assert_eq!(Rc::strong_count(&r), 1);
}
