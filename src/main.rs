use std::thread;
use crossbeam_channel::unbounded;

struct Buffer {
    id: i64,
    sub_array: Vec<i64>,
}

fn main() {
    let (work_queue_tx, work_queue_rx) = unbounded();
    let (ready_queue_tx, ready_queue_rx) = unbounded();
    let mut children = Vec::new();

    let T = 2; // number of threads in thread pool
    let mut vecs = vec![vec![1,1,2,2,3,3], vec![2,2,4,4,6,6]];
    let N = vecs.len();

    for t in 0..T {
        let rx = work_queue_rx.clone();
        let tx = ready_queue_tx.clone();
        children.push(thread::spawn(move || {
            loop {
                let mut b: Buffer = rx.recv().unwrap();
                if b.id == -1 {
                    let done_buf = Buffer { id: -1, sub_array: Vec::new()};
                    tx.send(done_buf);
                    break;
                } else {
                    b.sub_array.dedup();
                    tx.send(b);
                }
            }
        }));
    }


    let mut i = 0;
    for mut v in vecs {
        let block_size = v.len()/T;
        println!("pushing v: {:?}", v);
        while !v.is_empty() {
            let b = v.splice(0..block_size, []).collect();
            let buf = Buffer {
                id: i,
                sub_array: b,
            };
            work_queue_tx.send(buf);
        }
        i += 1;
    }

    println!("Done pushing to work queue");

    for t in 0..T {
        // fill the work queue with -1 buffer's to signal all the threads when they are done
        let buf = Buffer {
            id: -1,
            sub_array: Vec::new(),
        };

        work_queue_tx.send(buf);
    }

    let mut final_vecs: Vec<Vec<i64>> = Vec::with_capacity(N);
    for i in 0..N {
        final_vecs.push(vec![]);
    }
    loop {
        let mut b: Buffer = ready_queue_rx.recv().unwrap();
        if b.id == -1 {
            break;
        } else {
            final_vecs[b.id as usize].append(&mut b.sub_array);
        }
    }

    println!("final vecs: ");
    for mut v in final_vecs {
        v.dedup();
        println!("{:?}", v);
    }
}
