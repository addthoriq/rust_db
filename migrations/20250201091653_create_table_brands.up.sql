-- Add up migration script here

CREATE TABLE brands
(
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL
);