create table users
(
    id          serial primary key,
    fingerprint bytea not null unique,
    name        text  not null,
    max_health  int   not null,
    health      int   not null
        check (length(fingerprint) = 32)
        check (max_health > 0)
        check (health >= 0)
        check (health <= max_health)
);