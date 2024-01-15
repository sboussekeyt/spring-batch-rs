CREATE TABLE IF NOT EXISTS person
(
    id           BIGSERIAL PRIMARY KEY,
    last_name    TEXT        NOT NULL,
    first_name   TEXT        NOT NULL
);

INSERT INTO person (last_name, first_name)
VALUES 
('Finnegan', 'Melton'),
('Brayan', 'Pruitt'),
('Kaitlyn', 'Simmons'),
('Kristen', 'Dougherty'),
('Gina', 'Patton'),
('Emiliano', 'Michael'),
('Zion', 'Singh'),
('Kaydence', 'Morales'),
('Randy', 'Hull'),
('Daphne', 'Crosby'),
('Christopher', 'Gates'),
('Melina', 'Colon'),
('Nathan', 'Alvarado'),
('Mareli', 'Blackwell'),
('Kian', 'Lara'),
('Cory', 'Montes'),
('Iyana', 'Larson'),
('Sasha', 'Gentry');