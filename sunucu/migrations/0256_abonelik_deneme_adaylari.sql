-- B7: deneme süresi bitmek üzere olan çalışma alanlarını listeler (çalışma alanları arası).
--
-- Yalnız aday kimlikleri döner: deneme bitişi `created_at` artı kipin süresidir (billing::access
-- `trial_end`). Hatırlatma günü ve geçerli hak denetimi worker'da, çalışma alanı bağlamında yapılır.
-- `workspaces` tablosunda app_owner için `owner_list` politikası vardır (0101).
SET LOCAL ROLE app_owner;

CREATE FUNCTION billing_trial_candidates(
    p_now timestamptz, p_home_months integer, p_business_months integer,
    p_limit integer DEFAULT 1000)
    RETURNS TABLE (workspace_id uuid, created_at timestamptz, mode text)
    LANGUAGE sql STABLE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    AS $$
    SELECT w.id, w.created_at, w.mode FROM workspaces w
    WHERE w.deleted_at IS NULL
      AND w.created_at + make_interval(months => CASE WHEN w.mode = 'home'
                                                      THEN p_home_months
                                                      ELSE p_business_months END)
          BETWEEN p_now AND p_now + interval '8 days'
    LIMIT p_limit
$$;
REVOKE ALL ON FUNCTION billing_trial_candidates(timestamptz, integer, integer, integer)
    FROM PUBLIC;
GRANT EXECUTE ON FUNCTION billing_trial_candidates(timestamptz, integer, integer, integer)
    TO app_worker;

RESET ROLE;
