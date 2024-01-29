create table if not exists users (
	id serial primary key,
	name varchar not null,
	age int not null
);
