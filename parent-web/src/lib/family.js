import { supabase } from "./supabase.js";

/**
 * Ensure the signed-in parent has a family, returning its id.
 * Uses the create_family RPC (atomic: creates family + owner membership).
 */
export async function ensureFamily() {
  const { data: memberships, error } = await supabase
    .from("family_members")
    .select("family_id")
    .limit(1);
  if (error) throw error;
  if (memberships && memberships.length > 0) return memberships[0].family_id;

  const { data: familyId, error: rpcErr } = await supabase.rpc("create_family", {
    p_name: "My Family",
  });
  if (rpcErr) throw rpcErr;
  return familyId;
}

export async function addChild(name, birthYear) {
  const familyId = await ensureFamily();
  const { data, error } = await supabase
    .from("children")
    .insert({ family_id: familyId, name, birth_year: birthYear || null })
    .select("id, name, birth_year")
    .single();
  if (error) throw error;

  // Create a default (permissive) active policy for the new child.
  await supabase.from("policies").insert({ child_id: data.id, is_active: true });
  return data;
}
