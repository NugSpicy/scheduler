# Scheduler

## API Router

GET /task/:id
GET /task?state=&since=&until=
POST /task
  { type, send_ts }
DELETE /task/:id

Error {
  Code (Lifted into HTTP status code)
  Message
  Type
  Details
}

Shared State {
  DB Connection
}

## DB Connection

Scylla DB - Scalable distributed DB similar to Cassandra (interop: Cassandra, DynamoDB)
Task CR~~U~~D
 - create_task, read_task, read_tasks, delete_task
Task Procedure Call - Update then Read (Grab only rows you updated)
 - Lightweight Transaction (Prepared Transaction?)

Task {
  Id: UUID,
  Type: String, (enum)
  Send_ts: DateTime,
  State: String, (enum)
  Processor: UUID
}

## Schedule Worker

Grab BATCH_SIZE rows via updating their send_at time + 10s (2x timeout)
Process all tasks in parallel
Kill anything unfinished at 5s
Repeat

## Constraints

Exactly Once Delivery
Horizontal Scalability
API Contract Followed
-
Fault Tolerant
Error Tolerant

## Testing Considerations

Router Contract Testing
Parallelization Testing
Fault Tolerance Testing

## File Structure

/src
- /adapters (Scylla DB Adapter)
- - scylla.rs
- /models (Persistent Data Models + CR~~U~~D)
- - task.rs
- /procedures (Scylla Init CQL Script)
- - schema.cql
- main.rs
- worker.rs

# Specification

## Task Scheduler

We’re going to build a small task scheduling service. The service consists of an API listener, which accepts HTTP API calls, and a worker which executes tasks of different types. There are 3 task types: "Foo", "Bar", and "Baz".
- For "Foo" tasks, the worker should sleep for 3 seconds, and then print "Foo {task_id}".
- For "Bar" tasks, the worker should make a GET request to https://www.whattimeisitrightnow.com/ and print the response's status code
- For "Baz" tasks, the worker should generate a random number, N (0 to 343 inclusive), and print "Baz {N}"

## Requirements

Expose an API that can:
- Create a task of a specific type and execution time, returning the task's ID
- Show a list of tasks, filterable by their state (whatever states you define) and/or their task type
- Show a task based on its ID
- Delete a task based on its ID

The tasks must be persisted into some external data store.
Only external services allowed: a database of your choice.
Process each task only once and only at/after their specified execution time.

Support running multiple instances of your code in parallel, such that a cloud deployment could be horizontally scaled.

Open a PR against an empty repository that you create.
