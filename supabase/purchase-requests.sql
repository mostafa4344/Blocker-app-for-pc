-- Purchase requests for the manual (bKash/Nagad) buy flow.
-- Run this in the Supabase SQL editor once (after setup.sql).
--
-- The public purchase page INSERTs a row here with the anon/publishable key.
-- Buyers can ONLY insert; they cannot read, edit or delete any request (so one
-- buyer can never see another's details). You review requests in the Supabase
-- Table editor, verify the bKash/Nagad payment, then grant premium with
--   select public.admin_grant_premium('<install_id>', <days or null for lifetime>);
-- (null days = lifetime; 30 = monthly; 365 = yearly) and mark the row approved.

create table if not exists public.purchase_requests (
  id          uuid primary key default gen_random_uuid(),
  created_at  timestamptz not null default now(),
  install_id  text not null check (char_length(install_id) between 8 and 128),
  name        text not null check (char_length(name) between 1 and 120),
  email       text check (char_length(coalesce(email,'')) <= 200),
  phone       text check (char_length(coalesce(phone,'')) <= 40),
  plan        text not null check (plan in ('monthly','yearly','lifetime')),
  method      text not null check (method in ('bkash','nagad','other')),
  txn_id      text not null check (char_length(txn_id) between 3 and 80),
  status      text not null default 'pending' check (status in ('pending','approved','rejected')),
  note        text
);

create index if not exists purchase_requests_status_idx
  on public.purchase_requests (status, created_at desc);

alter table public.purchase_requests enable row level security;

-- No direct table access for the app: requests come in only through
-- submit_purchase_request() (see migrations/20261025070000_easy_premium.sql),
-- which checks the device, the plan and spam limits.
drop policy if exists "anon insert purchase requests" on public.purchase_requests;
revoke all on table public.purchase_requests from anon, authenticated;
