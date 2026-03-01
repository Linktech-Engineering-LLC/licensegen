CREATE TABLE IF NOT EXISTS zipcodes (
  `zip` int(5) unsigned zerofill NOT NULL,
  `city` varchar(64) NOT NULL,
  `state` char(2) NOT NULL,
  `county` varchar(28) DEFAULT NULL,
  PRIMARY KEY (`zip`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci |
