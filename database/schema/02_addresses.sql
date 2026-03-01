CREATE TABLE IF NOT EXISTS addresses (
   `id` bigint(20) unsigned NOT NULL,
  `maildrop` varchar(50) DEFAULT NULL,
  `street` varchar(50) DEFAULT NULL,
  `suite` varchar(50) DEFAULT NULL,
  `zip` int(5) unsigned zerofill NOT NULL,
  `zip4` int(4) unsigned zerofill DEFAULT NULL,
  `city` varchar(50) DEFAULT NULL,
  `state` varchar(50) DEFAULT NULL,
  `country` varchar(10) DEFAULT 'USA',
  `created` timestamp NOT NULL DEFAULT current_timestamp(),
  `updated` timestamp NOT NULL DEFAULT current_timestamp() ON UPDATE current_timestamp(),
  PRIMARY KEY (`id`) USING BTREE,
  KEY `FK_address_zipcodes` (`zip`),
  CONSTRAINT `FK_address_zipcodes` FOREIGN KEY (`zip`) REFERENCES `zipcodes` (`zip`) ON UPDATE CASCADE,
  CONSTRAINT `CC_MailStreet` CHECK (`maildrop` is not null or `street` is not null)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci |
