SELECT
    inflections
FROM
    '{{ table }}'
WHERE
    "case" = '{{ case }}'
    AND gender = '{{ gender }}'
    AND "number" = '{{ number }}'
