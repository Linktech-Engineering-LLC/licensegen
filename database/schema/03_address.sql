CREATE TABLE IF NOT EXISTS addresses (
  `id` bigint(20) unsigned NOT NULL,
  `maildrop` varchar(50) DEFAULT NULL COMMENT 'Post Office Box',
  `street` varchar(50) DEFAULT NULL,
  `suite` varchar(50) DEFAULT NULL,
  `zip` varchar(10) NOT NULL,
  `city` varchar(50) DEFAULT NULL,
  `state` varchar(50) DEFAULT NULL,
  `county` varchar(50) DEFAULT NULL,
  `country` varchar(10) NOT NULL,
  `created` timestamp NOT NULL DEFAULT current_timestamp(),
  `updated` timestamp NOT NULL DEFAULT current_timestamp() ON UPDATE current_timestamp(),
  PRIMARY KEY (`id`) USING BTREE,
  CONSTRAINT `CC_MailStreet` CHECK (`maildrop` is not null or `street` is not null)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci |
