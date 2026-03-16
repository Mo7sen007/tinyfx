use std::thread;
use anyhow::Result;

/// Runs a set of jobs in parallel across a fixed number of worker threads.
/// Collects all results. If any job returns an error, returns that error
/// after all threads have finished.
pub fn run_parallel<T, F>(jobs: Vec<F>, num_threads: usize) -> Result<Vec<T>>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T> + Send + 'static,
{
    let num_threads = num_threads.min(jobs.len()).max(1);
    let (tx, rx) = std::sync::mpsc::channel::<Result<T>>();

    // divide jobs into chunks — one chunk per thread
    let chunks: Vec<Vec<F>> = chunk_jobs(jobs, num_threads);

    thread::scope(|s| {
        for chunk in chunks {
            let tx = tx.clone();
            s.spawn(move || {
                for job in chunk {
                    let result = job();
                    // if send fails the receiver is gone — nothing we can do
                    let _ = tx.send(result);
                }
            });
        }
        // drop the original tx so the channel closes when all threads finish
        drop(tx);
    });

    // collect results — channel is closed so this drains completely
    let results: Vec<Result<T>> = rx.into_iter().collect();

    // If any job returned an error, propagate the first one we encounter.
    let mut ok_vals = Vec::with_capacity(results.len());
    for r in results {
        match r {
            Ok(v) => ok_vals.push(v),
            Err(e) => return Err(e),
        }
    }

    Ok(ok_vals)
}

fn chunk_jobs<T>(jobs: Vec<T>, num_chunks: usize) -> Vec<Vec<T>> {
    let mut chunks: Vec<Vec<T>> = (0..num_chunks).map(|_| Vec::new()).collect();
    for (i, job) in jobs.into_iter().enumerate() {
        chunks[i % num_chunks].push(job);
    }
    chunks
}