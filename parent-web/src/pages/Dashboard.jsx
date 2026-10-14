import { useEffect, useState } from "react";
import { supabase } from "../lib/supabase.js";

/**
 * Dashboard shell: lists the parent's children and each device's live status.
 * RLS ensures only this parent's rows come back. Policy editor, enrollment QR,
 * and reports are TODO (phase 1) — see docs/05-ROADMAP.md.
 */
export default function Dashboard() {
  const [children, setChildren] = useState([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    (async () => {
      // RLS scopes this to the signed-in parent's families automatically.
      const { data } = await supabase
        .from("children")
        .select("id, name, birth_year, devices(id, label, last_seen_at, battery_pct, foreground_app, revoked_at)")
        .order("name");
      setChildren(data ?? []);
      setLoading(false);
    })();
  }, []);

  async function signOut() {
    await supabase.auth.signOut();
  }

  return (
    <div style={{ maxWidth: 720, margin: "40px auto", fontFamily: "system-ui" }}>
      <header style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <h1>FamilyGuard</h1>
        <button onClick={signOut}>Sign out</button>
      </header>

      {loading ? <p>Loading…</p> : children.length === 0 ? (
        <EmptyState />
      ) : (
        children.map((c) => <ChildCard key={c.id} child={c} />)
      )}

      <p style={{ color: "#888", marginTop: 40, fontSize: 13 }}>
        Transparency: everything shown here is also disclosed to your child on their device.
      </p>
    </div>
  );
}

function EmptyState() {
  return (
    <div style={{ padding: 24, border: "1px dashed #ccc", borderRadius: 12 }}>
      <h3>Add your first child</h3>
      <p style={{ color: "#555" }}>
        Create a child profile, then enroll their phone by installing the FamilyGuard app and scanning
        the setup code. (This flow is coming in phase 1.)
      </p>
    </div>
  );
}

function ChildCard({ child }) {
  return (
    <div style={{ padding: 16, border: "1px solid #eee", borderRadius: 12, margin: "12px 0" }}>
      <h3 style={{ margin: 0 }}>{child.name}</h3>
      {(child.devices ?? []).map((d) => {
        const silent = d.last_seen_at
          ? (Date.now() - new Date(d.last_seen_at).getTime()) > 15 * 60 * 1000
          : true;
        return (
          <div key={d.id} style={{ marginTop: 8, fontSize: 14 }}>
            <strong>{d.label}</strong> —{" "}
            {d.revoked_at ? "Unlinked"
              : silent ? <span style={{ color: "#b00" }}>⚠ Not checked in recently</span>
              : <span style={{ color: "#070" }}>● Active</span>}
            {d.battery_pct != null && <> · 🔋 {d.battery_pct}%</>}
            {d.foreground_app && <> · {d.foreground_app}</>}
          </div>
        );
      })}
      {(child.devices ?? []).length === 0 && (
        <p style={{ color: "#888", fontSize: 14 }}>No device enrolled yet.</p>
      )}
    </div>
  );
}
