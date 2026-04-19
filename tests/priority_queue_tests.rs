//! Unit tests for priority queue functionality

use priority_queue::PriorityQueue;
use std::collections::HashMap;
use uuid::Uuid;

/// Priority levels for task scheduling
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum TaskPriority {
    NORMAL = 0,
    HIGH = 1,
}

/// Mock task for testing queue operations
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct MockTask {
    id: Uuid,
    name: String,
}

/// Test that priority queue correctly orders tasks by priority
#[test]
fn test_priority_queue_orders_by_priority() {
    let mut queue: PriorityQueue<Uuid, TaskPriority> = PriorityQueue::new();

    let task1 = Uuid::new_v4();
    let task2 = Uuid::new_v4();
    let task3 = Uuid::new_v4();

    // Add tasks with different priorities
    queue.push(task1, TaskPriority::NORMAL);
    queue.push(task2, TaskPriority::HIGH);
    queue.push(task3, TaskPriority::NORMAL);

    // HIGH priority should come first
    let (next_id, priority) = queue.peek().unwrap();
    assert_eq!(*priority, TaskPriority::HIGH);
    assert_eq!(*next_id, task2);
}

/// Test that tasks with same priority maintain FIFO order
#[test]
fn test_priority_queue_fifo_for_same_priority() {
    let mut queue: PriorityQueue<Uuid, TaskPriority> = PriorityQueue::new();

    let task1 = Uuid::new_v4();
    let task2 = Uuid::new_v4();
    let task3 = Uuid::new_v4();

    // Add multiple NORMAL priority tasks
    queue.push(task1, TaskPriority::NORMAL);
    queue.push(task2, TaskPriority::NORMAL);
    queue.push(task3, TaskPriority::NORMAL);

    // First added should be first to be processed
    let (next_id, _) = queue.peek().unwrap();
    assert_eq!(*next_id, task1);
}

/// Test task removal from queue
#[test]
fn test_priority_queue_remove_task() {
    let mut queue: PriorityQueue<Uuid, TaskPriority> = PriorityQueue::new();

    let task1 = Uuid::new_v4();
    let task2 = Uuid::new_v4();

    queue.push(task1, TaskPriority::HIGH);
    queue.push(task2, TaskPriority::NORMAL);

    // Remove specific task
    let removed = queue.remove(&task1);
    assert!(removed.is_some());

    // Verify queue still works
    let (next_id, _) = queue.peek().unwrap();
    assert_eq!(*next_id, task2);
}

/// Test that removing non-existent task returns None
#[test]
fn test_priority_queue_remove_nonexistent() {
    let mut queue: PriorityQueue<Uuid, TaskPriority> = PriorityQueue::new();

    let task1 = Uuid::new_v4();
    let nonexistent = Uuid::new_v4();

    queue.push(task1, TaskPriority::NORMAL);

    let removed = queue.remove(&nonexistent);
    assert!(removed.is_none());
}

/// Test queue length operations
#[test]
fn test_priority_queue_length() {
    let mut queue: PriorityQueue<Uuid, TaskPriority> = PriorityQueue::new();

    assert_eq!(queue.len(), 0);
    assert!(queue.is_empty());

    queue.push(Uuid::new_v4(), TaskPriority::NORMAL);
    assert_eq!(queue.len(), 1);
    assert!(!queue.is_empty());

    queue.push(Uuid::new_v4(), TaskPriority::HIGH);
    assert_eq!(queue.len(), 2);
}

/// Test queue clearing
#[test]
fn test_priority_queue_clear() {
    let mut queue: PriorityQueue<Uuid, TaskPriority> = PriorityQueue::new();

    queue.push(Uuid::new_v4(), TaskPriority::NORMAL);
    queue.push(Uuid::new_v4(), TaskPriority::HIGH);
    queue.push(Uuid::new_v4(), TaskPriority::NORMAL);

    queue.clear();

    assert_eq!(queue.len(), 0);
    assert!(queue.is_empty());
    assert!(queue.peek().is_none());
}

/// Test high priority task jumps ahead of many normal tasks
#[test]
fn test_priority_queue_high_priority_jumps_queue() {
    let mut queue: PriorityQueue<Uuid, TaskPriority> = PriorityQueue::new();

    // Add 100 normal priority tasks
    let mut normal_tasks = Vec::new();
    for i in 0..100 {
        let id = Uuid::new_v4();
        normal_tasks.push(id);
        queue.push(id, TaskPriority::NORMAL);
    }

    // Add one high priority task
    let urgent_task = Uuid::new_v4();
    queue.push(urgent_task, TaskPriority::HIGH);

    // High priority should be at front despite being added last
    let (next_id, priority) = queue.peek().unwrap();
    assert_eq!(*priority, TaskPriority::HIGH);
    assert_eq!(*next_id, urgent_task);
}

/// Test changing priority of existing task
#[test]
fn test_priority_queue_change_priority() {
    let mut queue: PriorityQueue<Uuid, TaskPriority> = PriorityQueue::new();

    let task1 = Uuid::new_v4();
    let task2 = Uuid::new_v4();

    queue.push(task1, TaskPriority::NORMAL);
    queue.push(task2, TaskPriority::NORMAL);

    // Change priority of task1 to HIGH
    queue.change_priority(&task1, TaskPriority::HIGH);

    // Now task1 should be first
    let (next_id, _) = queue.peek().unwrap();
    assert_eq!(*next_id, task1);
}

/// Test pop operation removes and returns highest priority task
#[test]
fn test_priority_queue_pop() {
    let mut queue: PriorityQueue<Uuid, TaskPriority> = PriorityQueue::new();

    let task1 = Uuid::new_v4();
    let task2 = Uuid::new_v4();

    queue.push(task1, TaskPriority::NORMAL);
    queue.push(task2, TaskPriority::HIGH);

    let popped = queue.pop();
    assert!(popped.is_some());
    let (id, priority) = popped.unwrap();
    assert_eq!(id, task2);
    assert_eq!(priority, TaskPriority::HIGH);
    assert_eq!(queue.len(), 1);
}

/// Test iterator over queue elements
#[test]
fn test_priority_queue_iteration() {
    let mut queue: PriorityQueue<Uuid, TaskPriority> = PriorityQueue::new();

    let task1 = Uuid::new_v4();
    let task2 = Uuid::new_v4();
    let task3 = Uuid::new_v4();

    queue.push(task1, TaskPriority::HIGH);
    queue.push(task2, TaskPriority::NORMAL);
    queue.push(task3, TaskPriority::HIGH);

    // Collect all tasks
    let tasks: Vec<_> = queue.iter().collect();
    assert_eq!(tasks.len(), 3);

    // Verify all 3 tasks are present (iteration order depends on internal heap structure)
    let ids: Vec<_> = tasks.iter().map(|(id, _)| **id).collect();
    assert!(ids.contains(&task1));
    assert!(ids.contains(&task2));
    assert!(ids.contains(&task3));

    // Verify peek returns highest priority (iteration != peek order)
    let (peek_id, peek_priority) = queue.peek().unwrap();
    assert_eq!(*peek_priority, TaskPriority::HIGH);
    assert!(*peek_id == task1 || *peek_id == task3);
}

/// Test queue with complex custom types
#[test]
fn test_priority_queue_with_complex_types() {
    let mut queue: PriorityQueue<MockTask, TaskPriority> = PriorityQueue::new();

    let task1 = MockTask {
        id: Uuid::new_v4(),
        name: "Task 1".to_string(),
    };
    let task2 = MockTask {
        id: Uuid::new_v4(),
        name: "Task 2".to_string(),
    };

    queue.push(task1.clone(), TaskPriority::NORMAL);
    queue.push(task2.clone(), TaskPriority::HIGH);

    let (next_task, priority) = queue.peek().unwrap();
    assert_eq!(next_task.name, "Task 2");
    assert_eq!(*priority, TaskPriority::HIGH);
}

/// Test queue behavior with HashMap for tracking
#[test]
fn test_priority_queue_with_task_tracking() {
    let mut queue: PriorityQueue<Uuid, TaskPriority> = PriorityQueue::new();
    let mut task_map: HashMap<Uuid, String> = HashMap::new();

    let task1 = Uuid::new_v4();
    let task2 = Uuid::new_v4();

    queue.push(task1, TaskPriority::HIGH);
    task_map.insert(task1, "High priority task".to_string());

    queue.push(task2, TaskPriority::NORMAL);
    task_map.insert(task2, "Normal task".to_string());

    // Process queue
    while let Some((id, _)) = queue.pop() {
        let description = task_map.get(&id).unwrap();
        assert!(description.contains("task"));
    }

    assert!(queue.is_empty());
}
