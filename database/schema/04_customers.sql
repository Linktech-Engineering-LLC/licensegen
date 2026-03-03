CREATE TABLE IF NOT EXISTS customers (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `company` varchar(255) DEFAULT NULL,
  `first` varchar(128) NOT NULL COMMENT 'First Name',
  `last` varchar(128) NOT NULL COMMENT 'Last Name',
  `email` varchar(255) NOT NULL,
  `phone` varchar(32) NOT NULL,
  `address_id` bigint(20) unsigned NOT NULL,
  `notes` text DEFAULT NULL,
  `created` timestamp NOT NULL DEFAULT current_timestamp(),
  `updated` timestamp NOT NULL DEFAULT current_timestamp() ON UPDATE current_timestamp(),
  PRIMARY KEY (`id`) USING BTREE,
  UNIQUE KEY `email` (`email`),
  KEY `FK_customers_address` (`address_id`),
  CONSTRAINT `FK_customers_address` FOREIGN KEY (`address_id`) REFERENCES `address` (`id`) ON UPDATE CASCADE
) ENGINE=InnoDB AUTO_INCREMENT=2 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci |
