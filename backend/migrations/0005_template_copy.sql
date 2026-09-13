UPDATE cv_templates
SET description = 'A compact, structured CV with strong section hierarchy.',
    updated_at = now()
WHERE id = 'editorial-v1'
  AND description = 'A compact, proof-oriented CV with strong section hierarchy.';
