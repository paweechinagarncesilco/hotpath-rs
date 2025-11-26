use futures_util::stream::{self, StreamExt};
use rand::Rng;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::sleep;

#[cfg_attr(feature = "hotpath", hotpath::measure)]
fn fast_sync_allocator() -> Vec<Vec<u64>> {
    let mut rng = rand::thread_rng();
    let num_arrays = rng.gen_range(1..=10);
    let mut arrays = Vec::new();

    for _ in 0..num_arrays {
        let size = rng.gen_range(10..100);
        let data: Vec<u64> = (0..size).map(|_| rng.gen()).collect();
        std::hint::black_box(&data);
        arrays.push(data);
    }

    std::thread::sleep(Duration::from_micros(rng.gen_range(10..50)));
    arrays
}

#[cfg_attr(feature = "hotpath", hotpath::measure)]
fn medium_sync_allocator() -> Vec<Vec<u64>> {
    let mut rng = rand::thread_rng();
    let num_arrays = rng.gen_range(1..=10);
    let mut arrays = Vec::new();

    for _ in 0..num_arrays {
        let size = rng.gen_range(100..1000);
        let data: Vec<u64> = (0..size).map(|_| rng.gen()).collect();
        std::hint::black_box(&data);
        arrays.push(data);
    }

    std::thread::sleep(Duration::from_micros(rng.gen_range(50..150)));
    arrays
}

#[cfg_attr(feature = "hotpath", hotpath::measure)]
fn slow_sync_allocator() -> Vec<Vec<u64>> {
    let mut rng = rand::thread_rng();
    let num_arrays = rng.gen_range(1..=10);
    let mut arrays = Vec::new();

    for _ in 0..num_arrays {
        let size = rng.gen_range(1000..10000);
        let data: Vec<u64> = (0..size).map(|_| rng.gen()).collect();
        std::hint::black_box(&data);
        arrays.push(data);
    }

    std::thread::sleep(Duration::from_micros(rng.gen_range(100..300)));
    arrays
}

#[cfg_attr(feature = "hotpath", hotpath::measure)]
async fn fast_async_allocator() -> Vec<Vec<u64>> {
    let mut rng = rand::thread_rng();
    let num_arrays = rng.gen_range(1..=10);
    let mut arrays = Vec::new();

    for _ in 0..num_arrays {
        let size = rng.gen_range(10..100);
        let data: Vec<u64> = (0..size).map(|_| rng.gen()).collect();
        std::hint::black_box(&data);
        arrays.push(data);
    }

    sleep(Duration::from_micros(rng.gen_range(10..50))).await;
    arrays
}

#[cfg_attr(feature = "hotpath", hotpath::measure)]
async fn slow_async_allocator() -> Vec<Vec<u64>> {
    let mut rng = rand::thread_rng();
    let num_arrays = rng.gen_range(1..=10);
    let mut arrays = Vec::new();

    for _ in 0..num_arrays {
        let size = rng.gen_range(1000..5000);
        let data: Vec<u64> = (0..size).map(|_| rng.gen()).collect();
        std::hint::black_box(&data);
        arrays.push(data);
    }

    sleep(Duration::from_micros(rng.gen_range(100..400))).await;
    arrays
}

/// Async function designed to migrate between threads.
/// Many yield points give the executor opportunities to reschedule on different workers.
#[cfg_attr(feature = "hotpath", hotpath::measure)]
async fn cross_thread_worker() -> u64 {
    let mut total = 0u64;

    // Many yield points to maximize chance of thread migration
    for i in 0..20 {
        tokio::task::yield_now().await;
        tokio::task::yield_now().await;
        tokio::task::yield_now().await;
        sleep(Duration::from_micros(1)).await;
        total += i;
    }

    total
}

/// Another async function with many awaits to demonstrate cross-thread behavior.
#[cfg_attr(feature = "hotpath", hotpath::measure)]
async fn heavy_async_work() -> Vec<u64> {
    let mut results = Vec::new();

    for _ in 0..10 {
        // CPU work
        let data: Vec<u64> = (0..100).map(|x| x * 2).collect();
        results.extend(data.iter().take(5));

        // Multiple yields per iteration
        tokio::task::yield_now().await;
        tokio::task::yield_now().await;
        sleep(Duration::from_micros(1)).await;
        tokio::task::yield_now().await;
    }

    results
}

#[cfg_attr(feature = "hotpath", hotpath::measure)]
fn process_data(arrays: Vec<Vec<u64>>) -> u64 {
    let mut rng = rand::thread_rng();
    let mut total_sum = 0u64;

    for data in arrays {
        let sum: u64 = data
            .iter()
            .take(rng.gen_range(5..20))
            .fold(0u64, |acc, &x| acc.wrapping_add(x % 1000));
        total_sum = total_sum.wrapping_add(sum);
    }

    std::hint::black_box(total_sum);
    total_sum
}

#[tokio::main]
#[cfg_attr(feature = "hotpath", hotpath::main)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting 60-second profiling test...");

    let (fast_tx, fast_rx) = mpsc::channel::<u64>(100);
    let (slow_tx, slow_rx) = mpsc::channel::<String>(50);

    #[cfg(feature = "hotpath")]
    let (fast_tx, fast_rx) =
        hotpath::channel!((fast_tx, fast_rx), label = "fast_metrics", log = true);
    #[cfg(feature = "hotpath")]
    let (slow_tx, slow_rx) = hotpath::channel!((slow_tx, slow_rx), label = "slow_events");

    let mut fast_rx = fast_rx;
    let mut slow_rx = slow_rx;

    let fast_stream = stream::iter(0u64..);
    let slow_stream = stream::iter(0u64..);

    #[cfg(feature = "hotpath")]
    let fast_stream = hotpath::stream!(fast_stream, label = "fast_metrics_stream", log = true);
    #[cfg(feature = "hotpath")]
    let slow_stream = hotpath::stream!(slow_stream, label = "slow_status_stream");

    // Pin the streams for consumption
    let mut fast_stream = Box::pin(fast_stream);
    let mut slow_stream = Box::pin(slow_stream);

    // Spawn fast channel consumer
    let fast_consumer = tokio::spawn(async move {
        let mut count = 0u64;
        while let Some(value) = fast_rx.recv().await {
            count = count.wrapping_add(value);
            if count.is_multiple_of(1000) {
                std::hint::black_box(count);
            }
        }
    });

    // Spawn slow channel consumer
    let slow_consumer = std::thread::spawn(move || {
        while let Some(msg) = slow_rx.blocking_recv() {
            std::hint::black_box(msg.len());
        }
    });

    let start = Instant::now();
    let duration = Duration::from_secs(60);
    let mut iteration = 0;

    while start.elapsed() < duration {
        iteration += 1;
        let elapsed = start.elapsed().as_secs();

        if iteration % 10 == 0 {
            println!(
                "[{:>2}s] Iteration {}: Calling mixed sync/async functions...",
                elapsed, iteration
            );
        }

        let mut rng = rand::thread_rng();

        // Send data to fast channel frequently
        let _ = fast_tx.send(rng.gen()).await;

        // Send data to slow channel occasionally
        if iteration % 5 == 0 {
            let _ = slow_tx
                .send(format!("Event at iteration {}", iteration))
                .await;
        }

        // Consume from fast stream frequently
        if let Some(value) = fast_stream.next().await {
            std::hint::black_box(value);
        }

        // Consume from slow stream occasionally
        if iteration % 7 == 0 {
            if let Some(value) = slow_stream.next().await {
                std::hint::black_box(value);
            }
        }

        // Call allocator functions which now randomly allocate 1-10 arrays each
        // Run some sync functions on separate threads to show different TIDs
        let data1_task = tokio::task::spawn_blocking(fast_sync_allocator);
        let data2_task = tokio::task::spawn_blocking(medium_sync_allocator);

        if iteration % 3 == 0 {
            let data3_task = tokio::task::spawn_blocking(|| {
                let data = slow_sync_allocator();
                process_data(data)
            });
            let _ = data3_task.await;
        }

        let data4 = fast_async_allocator().await;
        let data4_task = tokio::task::spawn_blocking(move || process_data(data4));
        let _ = data4_task.await;

        if iteration % 2 == 0 {
            let data5 = slow_async_allocator().await;
            let data5_task = tokio::task::spawn_blocking(move || process_data(data5));
            let _ = data5_task.await;
        }

        // Call cross-thread async functions (may migrate between worker threads)
        // Spawn them as separate tasks to increase migration likelihood
        let cross1 = tokio::spawn(cross_thread_worker());
        let cross2 = tokio::spawn(cross_thread_worker());
        let cross3 = tokio::spawn(heavy_async_work());

        // Also call directly (will run on current worker but may migrate)
        let _ = cross_thread_worker().await;
        let _ = heavy_async_work().await;

        let _ = cross1.await;
        let _ = cross2.await;
        let _ = cross3.await;

        let data1 = data1_task.await.unwrap();
        let data1_process_task = tokio::task::spawn_blocking(move || process_data(data1));
        let _ = data1_process_task.await;

        if iteration % 4 == 0 {
            let data2 = data2_task.await.unwrap();
            let data2_process_task = tokio::task::spawn_blocking(move || process_data(data2));
            let _ = data2_process_task.await;
        } else {
            // Still need to consume data2_task to avoid leaking it
            let _ = data2_task.await;
        }

        sleep(Duration::from_millis(rng.gen_range(10..50))).await;

        #[cfg(feature = "hotpath")]
        hotpath::measure_block!("iteration_block", {
            let temp: Vec<u32> = (0..rng.gen_range(50..200)).map(|_| rng.gen()).collect();
            std::hint::black_box(&temp);
        });
    }

    // Close channels
    drop(fast_tx);
    drop(slow_tx);

    // Wait for consumers to finish
    let _ = fast_consumer.await;

    slow_consumer.join().unwrap();

    Ok(())
}
