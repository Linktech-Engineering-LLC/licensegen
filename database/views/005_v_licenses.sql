CREATE OR REPLACE VIEW v_licenses AS
SELECT
    l.id AS id,
    COALESCE(l.`version`, "") AS `version`,
    COALESCE(l.payload, "") AS payload,
    COALESCE(l.features, "") AS features,
    COALESCE(l.signature, "") AS signature,
    l.issued AS issued,
    COALESCE(l.expires, DATE_ADD(l.issued, INTERVAL 1 YEAR)) AS expires,
    l.revoked as revoked,
    
    -- Application fields
    a.application_name AS application_name,
    a.application_price AS application_price,
    a.major AS major,
    a.validity_value AS validity_value,
    a.validity_unit AS validity_unit,

    -- Customer fields
    a.company AS company,
    a.first AS first,
    a.last AS last,

    -- Edition fields
    a.product_name AS product_name,
    a.edition_name AS edition_name,
    a.sku AS sku

FROM licenses l
JOIN v_applications a
    ON l.application_id = a.id;		