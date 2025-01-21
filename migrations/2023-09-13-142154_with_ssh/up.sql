create table ssh (
	id integer primary key not null,
         name text not null unique,
	      pw_name text not null,
        user varchar not null,
        host varchar not null
);


create table ssh_projects (
    id integer primary key not null,
    project_id integer not null references projects(id),
    ssh_id integer not null references ssh(id))
