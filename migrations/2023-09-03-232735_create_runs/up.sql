create table scripts (
	id integer primary key not null,
         name text not null unique,
	 path text not null,
         language_id integer not null references languages(id)
         );

create table scripts_lang_defaults(
	id integr primary key not null,
	language_id integer not null references languages(id),
	script_id integer not null references scripts(id)
)

