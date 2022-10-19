use std::arch::asm;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{hint, thread};
use std::thread::*;
use std::sync::Mutex;

fn acquire_spinlock_arm(old_val: &Arc<AtomicUsize>)
{
    let old_clone = Arc::clone(old_val);
    loop 
    {
        if old_clone.compare_exchange(0, 1, Ordering::Relaxed, Ordering::Acquire) == Ok(0)
        {
            break;
        } 
        for _ in 0..1024
        {
            //thread::yield_now();
            unsafe{ asm!("yield") };
            // unsafe{ asm!("pause")};
        }
    }

}

fn release_spinlock_arm(lockvar: &Arc<AtomicUsize>)
{
    let lock_clone = Arc::clone(lockvar);
    lock_clone.store(0, Ordering::Release);
}


static mut n_counter: usize = 0;
static THREAD_COUNT: usize = 8;

fn bench_mutex()
{
    let comparison_mutex = Arc::new(Mutex::new(0));
    let mut thread_hndls: Vec<JoinHandle<()>> = Vec::with_capacity(THREAD_COUNT);

    for _ in 0..THREAD_COUNT
    {   
        let mutex_clone = Arc::clone(&comparison_mutex);
        let thread = thread::spawn(move|| {

        for _ in 0..10000000
        {
            let mut num = mutex_clone.lock().unwrap();
            *num += 1;
        }
        });

        thread_hndls.push(thread);
    }

    for thrd in thread_hndls
    {
        if let Err(panic) = thrd.join() {
            println!("Thread had an error: {panic:?}");
        }
    }
    let mutex_clone2 = Arc::clone(&comparison_mutex);
    println!("{}", mutex_clone2.lock().unwrap());

}

unsafe fn bench_spinlock()
{
    let spinlock = Arc::new(AtomicUsize::new(0));
    let mut thread_hndls: Vec<JoinHandle<()>> = Vec::with_capacity(THREAD_COUNT);
    
    for _ in 0..THREAD_COUNT
    {
        let clone = Arc::clone(&spinlock);
        let thread = thread::spawn(move|| {
            
                for _ in 0..10000000
                {
                    acquire_spinlock_arm(&clone);
                    n_counter += 1;
                    release_spinlock_arm(&clone);
                }
                
        });
        thread_hndls.push(thread);
    }

    for thrd in thread_hndls
    {
        if let Err(panic) = thrd.join() {
            println!("Thread had an error: {panic:?}");
        }
    }
    
}
fn main() {
    unsafe {
        use std::time::Instant;
        let now = Instant::now();
         bench_spinlock();
         println!("{}", n_counter);
         let elapsed = now.elapsed();
         println!("스핀락 소요시간: {:.2?}", elapsed);  
    }

    {
        use std::time::Instant;
        let now = Instant::now();
         bench_mutex();
         let elapsed = now.elapsed();
         println!("뮤텍스 소요시간: {:.2?}", elapsed);  
    }
}
