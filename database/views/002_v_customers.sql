CREATE OR REPLACE VIEW v_customers AS
SELECT 
    c.id AS id,
    COALESCE(c.company, "") AS company,
    COALESCE(c.first, "") AS first,
    COALESCE(c.last, "") AS last,
    c.email AS email,
    COALESCE(c.phone, "") AS phone,
    COALESCE(c.notes, "") as notes,
    a.address_id as address_id
FROM customers c;