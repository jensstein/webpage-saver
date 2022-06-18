begin;
-- Can be extended later with `alter type user_role add value 'value';
-- https://stackoverflow.com/a/7834949
create type user_role as enum ('admin', 'user');

alter table users add column roles user_role array;
update users set roles = '{user}' where roles is null;
alter table users alter column roles set not null;
commit;
