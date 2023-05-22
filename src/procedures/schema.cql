CREATE KEYSPACE IF NOT EXISTS scheduler
    WITH REPLICATION = { 'class' : 'SimpleStrategy', 'replication_factor' : 1 };

CREATE TABLE IF NOT EXISTS scheduler.tasks (
    partition   int,
    id          uuid,
    task_type   text,
    send_ts     timestamp,
    state       text,
    processor   uuid,

    PRIMARY KEY ((partition), id)
);
