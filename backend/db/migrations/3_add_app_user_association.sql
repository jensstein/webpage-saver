-- This table holds links between a user and an app authorized via oauth2.
-- The sub column is meant to hold values taken from the "sub" part of a jwt token.
create table connected_apps(
    id bigserial primary key,
    user_id bigint not null references users(id),
    sub text not null,
    client_id text not null,
    app_host text not null, -- where the app runs, e.g. Firefox on Linux. Both for display purposes and to enable handling of multiple apps per user.
    last_used timestamp with time zone
);
create index connected_app_user_id_idx on connected_apps (user_id);
create index connected_app_sub_idx on connected_apps (sub);
create unique index connected_app_user_id_app_host_unique_idx on connected_apps (user_id, app_host);
