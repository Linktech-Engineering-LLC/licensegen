CREATE OR REPLACE VIEW v_address AS
SELECT 
    a.id AS id,
    COALESCE(a.maildrop, "") AS maildrop,
    COALESCE(a.street, "") AS street,
    COALESCE(a.suite, "") AS suite,
    COALESCE(a.city, z.city, 'unknown') AS city,
    COALESCE(a.county, z.county, 'unknown') AS county,
    COALESCE(a.state, z.state, 'unknown') AS state,
    COALESCE(a.country, "") as country,
    a.zip AS zip
FROM address a
JOIN zipcodes z 
    ON (a.zip % 100000) = z.zip;