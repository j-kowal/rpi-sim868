use crate::{
    error::{Error, ErrorKind},
    ResolverReturn, TaskJoinHandle,
};
use colored::Colorize;
use priority_queue::PriorityQueue;
use rppal::uart::{Parity, Uart};
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::{spawn, sync::RwLock, time::sleep};
use uuid::Uuid;

const MUTEX_POISONED_MSG: &str = "Critical error: Mutex is poisoned.";

pub struct SerialPort {
    uart: Arc<Mutex<Uart>>,
    queue: Arc<RwLock<PriorityQueue<Uuid, TaskPriority>>>,
}

#[derive(PartialEq, PartialOrd, Ord, Eq, Debug)]
pub enum TaskPriority {
    NORMAL,
    HIGH,
}

fn debug_log(task_id: &Uuid, msg: &str) {
    log::debug!("{} - {msg}", format!("[{task_id}]").yellow())
}

fn info_log(task_id: &Uuid, msg: &str) {
    log::info!("{} - {msg}", format!("[{task_id}]").yellow())
}

async fn add_to_queue(serial_port: &Arc<SerialPort>, priority: TaskPriority) -> Uuid {
    let task_id: Uuid = Uuid::new_v4();
    debug_log(&task_id, &format!("created with {priority:?} priority."));
    serial_port.queue.write().await.push(task_id, priority);
    task_id
}

async fn await_in_queue(task_id: &Uuid, serial_port: &Arc<SerialPort>) {
    loop {
        let queue: tokio::sync::RwLockReadGuard<'_, PriorityQueue<Uuid, TaskPriority>> =
            serial_port.queue.read().await;
        let (next, _) = queue
            .peek()
            .expect("Critical error: task queue is corrupted.");
        if *next == *task_id {
            break;
        }

        drop(queue);
        sleep(Duration::from_millis(100)).await;
    }
}

async fn remove_from_queue(task_id: &Uuid, serial_port: &Arc<SerialPort>) {
    serial_port.queue.write().await.remove(&task_id);
    debug_log(task_id, "removed from the queue.");
}

fn uart_read<T>(
    task_id: &Uuid,
    uart: &mut std::sync::MutexGuard<'_, Uart>,
    timeout: Duration,
    resolver: fn(String) -> ResolverReturn<T>,
) -> ResolverReturn<T> {
    let mut data: Option<T> = None;
    let mut error: Option<Error> = None;
    let start: Instant = Instant::now();

    while start.elapsed() <= timeout {
        let mut read_vec: Vec<u8> = Vec::new();
        let mut read_buffer: [u8; 1] = [0];

        while uart.read(&mut read_buffer)? > 0 {
            read_vec.push(read_buffer[0]);
        }

        if !read_vec.is_empty() {
            debug_log(task_id, &format!("read vector: {read_vec:?}"));
        }

        let read: String = String::from_utf8(read_vec).unwrap_or("".to_string());
        if !read.is_empty() {
            debug_log(task_id, &format!("parsed string: {read}"));
        }

        match resolver(read) {
            Ok(d) => {
                debug_log(task_id, "resolved.");
                data = Some(d);
                break;
            }
            Err(e) => match e.kind() {
                ErrorKind::NotResolved => (),
                _ => {
                    error = Some(e);
                    break;
                }
            },
        }
    }

    if let Some(err) = error {
        log::error!("{} - error: {err:?}", format!("[{task_id}]").yellow());
        return Err(err);
    }

    match data {
        Some(data) => Ok(data),
        None => Err(Error::NotResolved),
    }
}

pub fn spawn_task<T1, T2>(
    serial_port: Arc<SerialPort>,
    priority: TaskPriority,
    task_fn: fn(&Arc<SerialPort>, &Uuid, T2) -> ResolverReturn<T1>,
    log_msg: Option<String>,
    arguments: T2,
) -> TaskJoinHandle<T1>
where
    T1: 'static + Send,
    T2: 'static + Send,
{
    spawn(async move {
        let task_id: Uuid = add_to_queue(&serial_port, priority).await;
        if let Some(msg) = log_msg {
            info_log(&task_id, &msg);
        }
        await_in_queue(&task_id, &serial_port).await;
        let result: Result<T1, Error> = task_fn(&serial_port, &task_id, arguments);
        remove_from_queue(&task_id, &serial_port).await;
        result
    })
}

impl SerialPort {
    pub fn new(path: &str, baud_rate: u32) -> Self {
        let mut uart: Uart = Uart::with_path(path, baud_rate, Parity::None, 8, 1)
            .expect("Unable to establish UART connection.");
        uart.set_read_mode(0, Duration::from_millis(100))
            .expect("Unable to set UART read mode.");

        SerialPort {
            uart: Arc::new(Mutex::new(uart)),
            queue: Arc::new(RwLock::new(PriorityQueue::new())),
        }
    }

    pub fn write(&self, task_id: &Uuid, input: String) -> ResolverReturn<()> {
        let mut uart: std::sync::MutexGuard<'_, Uart> = self.uart.lock().expect(MUTEX_POISONED_MSG);
        uart.flush(rppal::uart::Queue::Input)?;
        debug_log(task_id, "Writing to UART...");
        uart.write(input.as_bytes())?;
        Ok(())
    }

    pub fn read<T>(
        &self,
        task_id: &Uuid,
        resolver: fn(String) -> ResolverReturn<T>,
        timeout: Option<Duration>,
    ) -> ResolverReturn<T> {
        let timeout: Duration = timeout.unwrap_or(Duration::from_millis(1000));
        let mut uart: std::sync::MutexGuard<'_, Uart> = self.uart.lock().expect(MUTEX_POISONED_MSG);
        let read: ResolverReturn<T> = uart_read(&task_id, &mut uart, timeout, resolver);
        read
    }

    pub fn process<T>(
        &self,
        task_id: &Uuid,
        input: String,
        resolver: fn(String) -> ResolverReturn<T>,
        timeout: Option<Duration>,
    ) -> ResolverReturn<T> {
        let timeout: Duration = timeout.unwrap_or(Duration::from_millis(1000));
        let mut uart: std::sync::MutexGuard<'_, Uart> = self.uart.lock().expect(MUTEX_POISONED_MSG);
        uart.flush(rppal::uart::Queue::Both)?;
        uart.write(input.as_bytes())?;
        let read: ResolverReturn<T> = uart_read(task_id, &mut uart, timeout, resolver);
        read
    }
}
