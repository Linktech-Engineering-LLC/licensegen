CREATE TABLE IF NOT EXISTS customers (
  `id` bigint(20) unsigned NOT NULL,
  `company` varchar(255) NOT NULL,
  `first_name` varchar(128) NOT NULL,
  `last_name` varchar(128) NOT NULL,
  `email` varchar(255) NOT NULL,
  `phone` varchar(32) NOT NULL,
  `address_id` bigint(20) unsigned NOT NULL,
  `notes` text NOT NULL,
  `created` timestamp NOT NULL DEFAULT current_timestamp(),
  `updated` timestamp NOT NULL DEFAULT current_timestamp() ON UPDATE current_timestamp(),
  PRIMARY KEY (`id`) USING BTREE,
  UNIQUE KEY `email` (`email`),
  KEY `FK_customers_address` (`address_id`),
  CONSTRAINT `FK_customers_address` FOREIGN KEY (`address_id`) REFERENCES `address` (`id`) ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci |
