| CREATE TABLE IF NOT EXISTS `applications` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(255) NOT NULL,
  `customer_id` bigint(20) unsigned NOT NULL,
  `edition_id` bigint(20) unsigned NOT NULL,
  `raw_yaml` text NOT NULL,
  `received` date NOT NULL,
  `acquired` date NOT NULL,
  `status` enum('pending','approved','rejected') NOT NULL DEFAULT 'pending',
  `created` timestamp NOT NULL DEFAULT current_timestamp(),
  `updated` timestamp NOT NULL DEFAULT current_timestamp() ON UPDATE current_timestamp(),
  PRIMARY KEY (`id`) USING BTREE,
  UNIQUE KEY `name_customer__edition` (`name`,`customer_id`,`edition_id`) USING BTREE,
  KEY `FK_AppCust` (`customer_id`) USING BTREE,
  KEY `FK_AppProd` (`edition_id`) USING BTREE,
  CONSTRAINT `FK_AppCust` FOREIGN KEY (`customer_id`) REFERENCES `customers` (`id`) ON DELETE NO ACTION ON UPDATE NO ACTION,
  CONSTRAINT `FK_AppEdit` FOREIGN KEY (`edition_id`) REFERENCES `editions` (`id`) ON DELETE NO ACTION ON UPDATE NO ACTION
) ENGINE=InnoDB AUTO_INCREMENT=2 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci COMMENT='\r\n' |
