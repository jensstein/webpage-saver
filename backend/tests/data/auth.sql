START TRANSACTION;
INSERT INTO users (id, username, password_hash, roles) VALUES (1, 'user', '$argon2id$v=19$m=4096,t=3,p=1$VeLKNIaRldKePZQX1ZhxTw$1aiyGgI89hszm6CLNh9THrvmehKOBkqwatIUF6jq9Cg', '{user}');
INSERT INTO jwt_secrets (id, secret, user_id) VALUES (1, '\x4a7877684e576459654178246728414e537138597a5242486b4672265a5e2561257537552a68682838486347563559365636', 1);
INSERT INTO users (id, username, password_hash, roles) VALUES (2, 'admin', '$argon2id$v=19$m=4096,t=3,p=1$Eqd+3URmPgSNt8pchnc9KA$vJfubV6k4aPoO9+Qk7V3r02qIaXhJTQ0KOe1euC33g', '{user,admin}');
INSERT INTO jwt_secrets (id, secret, user_id) VALUES (2, '\x6d79713846524642484a7426746f282a65625341426329626d624b4669516a55706f4f564c5732767158706b6c254c355a54', 2);
select setval('users_id_seq', 3, true);
select setval('jwt_secrets_id_seq', 3, true);
END TRANSACTION;
