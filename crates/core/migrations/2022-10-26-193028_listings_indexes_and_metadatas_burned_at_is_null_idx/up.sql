CREATE INDEX IF NOT EXISTS listings_purchase_id_is_null_idx ON public.listings USING btree (purchase_id) WHERE purchase_id is null;
CREATE INDEX IF NOT EXISTS listings_canceled_at_is_null_idx ON public.listings USING btree (canceled_at) WHERE canceled_at is null;
CREATE INDEX IF NOT EXISTS metadatas_burned_at_is_null_idx ON public.metadatas USING btree (burned_at) WHERE burned_at is null;
CREATE INDEX IF NOT EXISTS listings_created_at_idx ON public.listings USING btree (created_at);
CREATE INDEX IF NOT EXISTS listings_expiry_is_null_idx ON public.listings USING btree (expiry) WHERE expiry is null;
CREATE INDEX IF NOT EXISTS listings_created_at_desc_idx ON public.listings USING btree (created_at DESC);
CREATE INDEX IF NOT EXISTS listings_created_at_asc_idx ON public.listings USING btree (created_at ASC);