CREATE TABLE IF NOT EXISTS `products` (
  `id` bigint(20) unsigned NOT NULL,
  `name` varchar(50) NOT NULL COMMENT 'product name',
  `code` varchar(8) NOT NULL COMMENT 'short code',
  `version` varchar(10) DEFAULT NULL COMMENT 'version string "n.n.n"',
  `editions` longtext CHARACTER SET utf8mb4 COLLATE utf8mb4_bin DEFAULT NULL COMMENT 'JSON of edition rules' CHECK (json_valid(`editions`)),
  `payload_schema` longtext CHARACTER SET utf8mb4 COLLATE utf8mb4_bin NOT NULL COMMENT 'JSON/YAML describing payload fields',
  `features` longtext CHARACTER SET utf8mb4 COLLATE utf8mb4_bin NOT NULL COMMENT 'JSON/YAML describing editions/features',
  `keypair_path` varchar(50) NOT NULL COMMENT 'Path to keypair',
  `active` tinyint(1) unsigned NOT NULL COMMENT 'whether product is active',
  `created` timestamp NOT NULL DEFAULT current_timestamp(),
  `updated` timestamp NOT NULL DEFAULT current_timestamp() ON UPDATE current_timestamp(),
  PRIMARY KEY (`id`) USING BTREE,
  UNIQUE KEY `name_code` (`name`,`code`),
  UNIQUE KEY `code` (`code`),
  UNIQUE KEY `name` (`name`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci |
