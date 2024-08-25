CREATE TABLE message
(
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TIMESTAMP     NOT NULL,
    name   VARCHAR(255)  NOT NULL,
    email     VARCHAR(255)  NOT NULL,
    contents  VARCHAR(1024) NOT NULL
);

INSERT INTO message (timestamp, name, email, contents)
VALUES ('2021-01-01 00:00:00', 'John Doe', 'john@doe.com', 'Hello, World!');

INSERT INTO message (timestamp, name, email, contents)
VALUES ('2021-01-02 00:00:00', 'Jane Doe', 'jane@doe.com', 'Hi, there!');

