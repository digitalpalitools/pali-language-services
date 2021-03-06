SELECT
    inflections
FROM
    '{{ table }}'
WHERE
    tense = '{{ tense }}'
    AND person = '{{ person }}'
    AND actreflx = '{{ actreflx }}'
    AND "number" = '{{ number }}'
