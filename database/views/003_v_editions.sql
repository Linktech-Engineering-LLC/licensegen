CREATE OR REPLACE VIEW v_editions AS
SELECT 
    e.id AS id,
    p.`name` AS product_name,
    COALESCE(p.version, "") as `version`,
    COALESCE(p.editions, "") as editions,
    p.payload_schema as payload_schema,
    p.features as features,
    p.keypair_path as keypair_path,
    p.active as active,
    e.`name` AS edition_name,
    e.sku AS sku,
    e.edition_code as edition_code,
    e.metadata as metadata,
    COALESCE(e.price, 0.00) AS price,
    COALESCE(e.valid, 0) AS valid
FROM editions e
JOIN products p 
    ON e.product_id = p.id;