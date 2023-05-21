CREATE KEYSPACE IF NOT EXISTS scheduler
    WITH REPLICATION = { 'class' : 'SimpleStrategy', 'replication_factor' : 1 };

CREATE TABLE IF NOT EXISTS scheduler.tasks (
    task_type   text,
    id          uuid PRIMARY KEY,
    send_at     timestamp,
    state       text,
    processor   uuid
);