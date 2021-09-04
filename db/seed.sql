create table locations
(
    id serial primary key
);

create table users
(
    id          serial primary key,
    fingerprint bytea not null unique,
    name        text  not null,
    max_health  int   not null,
    health      int   not null,
    location_id int   not null references locations
        check (length(fingerprint) = 32)
        check (max_health > 0)
        check (health >= 0)
        check (health <= max_health)
        check (location_id >= 0)
);

insert into locations (id)
values (0);

insert into locations (id)
values (1);