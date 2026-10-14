import { createClient } from "@supabase/supabase-js";

// Public anon key only. RLS (backend/supabase/policies.sql) is the real security
// boundary — never rely on the client to hide other families' data.
const url = import.meta.env.VITE_SUPABASE_URL;
const anonKey = import.meta.env.VITE_SUPABASE_ANON_KEY;

if (!url || !anonKey) {
  // Fail loud in dev so misconfig is obvious.
  console.warn("Supabase env not set — copy .env.example to .env and fill it in.");
}

export const supabase = createClient(url ?? "", anonKey ?? "");
export const FUNCTIONS_URL = import.meta.env.VITE_FUNCTIONS_URL ?? "";
