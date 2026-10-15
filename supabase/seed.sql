-- Example activation codes for testing. In production the payment webhook
-- inserts codes (with service-role access) after a successful purchase.
insert into public.codes (code, days) values
  ('PREMIUM-LIFETIME', null),
  ('PREMIUM-30D', 30),
  ('PREMIUM-365D', 365)
on conflict (code) do nothing;
