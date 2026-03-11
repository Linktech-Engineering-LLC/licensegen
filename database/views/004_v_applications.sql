CREATE OR REPLACE VIEW v_applications AS
SELECT 
    a.id AS id,
    a.name AS application_name,
    a.edition_id as edition_id,

    -- Customer fields (already COALESCE’d in v_customers)
    a.customer_id as customer_id,
    c.company AS company,
    c.first AS first,
    c.last AS last,

    -- Edition fields (already COALESCE’d in v_editions)
    e.product_name AS product_name,
    e.edition_name AS edition_name,
    e.sku AS sku,
    e.valid AS edition_valid,

    -- Application commercial terms
    COALESCE(a.price, 0.00) AS application_price,
    COALESCE(a.valid_major, 0) AS major,
    a.validity_value AS validity_value,
    a.validity_unit AS validity_unit,

    -- Application metadata
    a.raw_yaml AS raw_yaml,
    a.received AS received,
    a.acquired AS acquired,
    a.status AS status

FROM applications a
JOIN v_customers c 
    ON a.customer_id = c.id
JOIN v_editions e 
    ON a.edition_id = e.id;