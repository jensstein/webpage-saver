START TRANSACTION;
INSERT INTO users (id, username, password_hash, roles) VALUES (1, 'user', '$argon2id$v=19$m=4096,t=3,p=1$VeLKNIaRldKePZQX1ZhxTw$1aiyGgI89hszm6CLNh9THrvmehKOBkqwatIUF6jq9Cg', '{user}');
INSERT INTO jwt_secrets (id, secret, user_id) VALUES (1, '\x4a7877684e576459654178246728414e537138597a5242486b4672265a5e2561257537552a68682838486347563559365636', 1);
select setval('users_id_seq', 2, true);
select setval('jwt_secrets_id_seq', 2, true);
END TRANSACTION;
