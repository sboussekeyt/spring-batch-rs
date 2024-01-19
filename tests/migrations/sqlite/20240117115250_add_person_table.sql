CREATE TABLE IF NOT EXISTS person
(
    id           integer primary key AUTOINCREMENT,
    last_name    varchar(30) not null,
    first_name   varchar(30) not null
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