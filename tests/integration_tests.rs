//! Integration tests for async task scheduling with priority queue

use priority_queue::PriorityQueue;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Task priority levels matching the library
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[allow(clippy::upper_case_acronyms)]
pub enum TaskPriority {
    NORMAL = 0,
    HIGH = 1,
}

/// Mock serial port with priority queue for testing
struct MockSerialPort {
    queue: Arc<RwLock<PriorityQueue<Uuid, TaskPriority>>>,
    processed_tasks: Arc<RwLock<Vec<(Uuid, TaskPriority)>>>,
}

impl MockSerialPort {
    fn new() -> Self {
        MockSerialPort {
            queue: Arc::new(RwLock::new(PriorityQueue::new())),
            processed_tasks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    async fn add_task(&self, priority: TaskPriority) -> Uuid {
        let id = Uuid::new_v4();
        self.queue.write().await.push(id, priority);
        id
    }

    async fn process_next(&self) -> Option<(Uuid, TaskPriority)> {
        let mut queue = self.queue.write().await;
        if let Some((id, priority)) = queue.pop() {
            drop(queue);
            self.processed_tasks.write().await.push((id, priority));
            Some((id, priority))
        } else {
            None
        }
    }

    async fn get_queue_len(&self) -> usize {
        self.queue.read().await.len()
    }
}

/// Test that tasks are processed in priority order
#[tokio::test]
async fn test_task_priority_ordering() {
    let port = MockSerialPort::new();

    // Add tasks in mixed order
    let normal1 = port.add_task(TaskPriority::NORMAL).await;
    let normal2 = port.add_task(TaskPriority::NORMAL).await;
    let high1 = port.add_task(TaskPriority::HIGH).await;
    let normal3 = port.add_task(TaskPriority::NORMAL).await;
    let high2 = port.add_task(TaskPriority::HIGH).await;

    // Process all tasks
    let mut processed = Vec::new();
    while let Some((id, priority)) = port.process_next().await {
        processed.push((id, priority));
    }

    // Verify order: high priority first
    assert_eq!(processed.len(), 5);
    assert_eq!(processed[0].1, TaskPriority::HIGH);
    assert_eq!(processed[1].1, TaskPriority::HIGH);
    assert_eq!(processed[2].1, TaskPriority::NORMAL);
    assert_eq!(processed[3].1, TaskPriority::NORMAL);
    assert_eq!(processed[4].1, TaskPriority::NORMAL);
}

/// Test concurrent task addition
#[tokio::test]
async fn test_concurrent_task_addition() {
    let port = Arc::new(MockSerialPort::new());
    let mut handles = Vec::new();

    // Spawn multiple tasks that add to queue concurrently
    for i in 0..10 {
        let port_clone = Arc::clone(&port);
        let priority = if i % 2 == 0 {
            TaskPriority::HIGH
        } else {
            TaskPriority::NORMAL
        };

        let handle = tokio::spawn(async move {
            port_clone.add_task(priority).await;
        });

        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify queue has all tasks
    assert_eq!(port.get_queue_len().await, 10);
}

/// Test task processing with simulated work time
#[tokio::test]
async fn test_task_processing_with_timing() {
    let port = MockSerialPort::new();

    // Add tasks
    port.add_task(TaskPriority::NORMAL).await;
    port.add_task(TaskPriority::HIGH).await;

    let start = Instant::now();

    // Process first task (should be HIGH priority)
    let (id, priority) = port.process_next().await.unwrap();
    let first_duration = start.elapsed();

    assert_eq!(priority, TaskPriority::HIGH);
    assert!(first_duration < Duration::from_millis(100)); // Should be fast
}

/// Test task removal from queue
#[tokio::test]
async fn test_task_removal() {
    let port = MockSerialPort::new();

    let task1 = port.add_task(TaskPriority::HIGH).await;
    let task2 = port.add_task(TaskPriority::NORMAL).await;

    // Remove specific task
    {
        let mut queue = port.queue.write().await;
        queue.remove(&task1);
    }

    // Verify queue state
    assert_eq!(port.get_queue_len().await, 1);

    // Process next - should be task2
    let (id, _) = port.process_next().await.unwrap();
    assert_eq!(id, task2);
}

/// Test queue behavior with rapid add/remove cycles
#[tokio::test]
async fn test_rapid_queue_operations() {
    let port = Arc::new(MockSerialPort::new());

    // Rapidly add and remove tasks
    for _ in 0..100 {
        let id = port.add_task(TaskPriority::NORMAL).await;

        {
            let mut queue = port.queue.write().await;
            queue.remove(&id);
        }
    }

    // Queue should be empty
    assert_eq!(port.get_queue_len().await, 0);
}

/// Test priority queue with simulated AT command responses
#[tokio::test]
async fn test_at_command_priority_handling() {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    #[allow(clippy::upper_case_acronyms)]
    enum ATCmdPriority {
        NORMAL,
        URGENT, // For calls
    }

    let queue: Arc<RwLock<PriorityQueue<String, ATCmdPriority>>> =
        Arc::new(RwLock::new(PriorityQueue::new()));

    // Simulate incoming commands
    queue
        .write()
        .await
        .push("AT+CMGS=...".to_string(), ATCmdPriority::NORMAL);
    queue
        .write()
        .await
        .push("ATA".to_string(), ATCmdPriority::URGENT); // Answer call
    queue
        .write()
        .await
        .push("AT+CGNSINF".to_string(), ATCmdPriority::NORMAL);

    // First should be answer call (URGENT)
    let (cmd, priority) = queue.write().await.pop().unwrap();
    assert_eq!(cmd, "ATA");
    assert_eq!(priority, ATCmdPriority::URGENT);
}

/// Test queue fairness - tasks with same priority should all be processed
#[tokio::test]
async fn test_queue_fairness() {
    let port = MockSerialPort::new();
    let mut task_ids = Vec::new();

    // Add 50 NORMAL priority tasks
    for _ in 0..50 {
        let id = port.add_task(TaskPriority::NORMAL).await;
        task_ids.push(id);
    }

    // Process all and verify all tasks are completed
    let mut processed = Vec::new();
    while let Some((id, _)) = port.process_next().await {
        processed.push(id);
    }

    // All tasks should be processed (exact FIFO order depends on priority_queue implementation)
    assert_eq!(processed.len(), task_ids.len());
    // Verify all original tasks are in processed list
    for id in &task_ids {
        assert!(processed.contains(id), "Task {} should be processed", id);
    }
}

/// Test queue capacity and performance
#[tokio::test]
async fn test_queue_performance() {
    let port = Arc::new(MockSerialPort::new());
    let start = Instant::now();

    // Add many tasks
    let num_tasks = 1000;
    for i in 0..num_tasks {
        let priority = if i % 10 == 0 {
            TaskPriority::HIGH
        } else {
            TaskPriority::NORMAL
        };
        port.add_task(priority).await;
    }

    let add_duration = start.elapsed();

    // Process all tasks
    let mut processed_high = 0;
    while let Some((_, priority)) = port.process_next().await {
        if priority == TaskPriority::HIGH {
            processed_high += 1;
        }
    }

    let total_duration = start.elapsed();

    // Verify results
    assert_eq!(processed_high, 100); // Every 10th was HIGH
    assert_eq!(port.get_queue_len().await, 0);

    // Performance check (should complete within reasonable time)
    assert!(total_duration < Duration::from_secs(5));
}

/// Test error handling in async context
#[tokio::test]
async fn test_async_error_handling() {
    #[derive(Debug, Clone)]
    enum MockError {
        QueueFull,
        InvalidPriority,
    }

    async fn try_add_task(priority: i32) -> Result<Uuid, MockError> {
        if priority < 0 {
            return Err(MockError::InvalidPriority);
        }
        Ok(Uuid::new_v4())
    }

    // Test valid priority
    let result = try_add_task(1).await;
    assert!(result.is_ok());

    // Test invalid priority
    let result = try_add_task(-1).await;
    assert!(result.is_err());
    match result {
        Err(MockError::InvalidPriority) => (),
        _ => panic!("Expected InvalidPriority error"),
    }
}

/// Test timeout mechanism for task execution
#[tokio::test]
async fn test_task_timeout() {
    use tokio::time::{sleep, timeout, Duration};

    async fn slow_task() -> Result<(), ()> {
        sleep(Duration::from_secs(10)).await;
        Ok(())
    }

    // Task should timeout
    let result = timeout(Duration::from_millis(100), slow_task()).await;
    assert!(result.is_err());
}

/// Test cleanup of completed tasks
#[tokio::test]
async fn test_task_cleanup() {
    let port = Arc::new(MockSerialPort::new());

    // Add and process many tasks
    for _ in 0..100 {
        let port_clone = Arc::clone(&port);
        tokio::spawn(async move {
            port_clone.add_task(TaskPriority::NORMAL).await;
        });
    }

    // Small delay for tasks to be added
    sleep(Duration::from_millis(50)).await;

    // Process all
    while port.process_next().await.is_some() {}

    // Verify clean state
    assert_eq!(port.get_queue_len().await, 0);
}

use tokio::time::sleep;
