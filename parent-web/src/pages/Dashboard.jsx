import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { supabase } from "../lib/supabase.js";
import { addChild } from "../lib/family.js";

/**
 * Dashboard: lists the parent's children + each device's live status, and lets
 * them add a child. RLS scopes everything to the signed-in parent.
 */
export default function Dashboard() {
  const [children, setChildren] = useState([]);
  const [loading, setLoading] = useState(true);
  const [name, setName] = useState("");
  const [birthYear, setBirthYear] = useState("");
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState("");

  async function load() {
    const { data, error } = await supabase
      .from("children")
      .select("id, name, birth_year, devices(id, label, last_seen_at, battery_pct, foreground_app, revoked_at)")
      .order("name");
    if (error) setErr(error.message);
    setChildren(data ?? []);
    setLoading(false);
  }

  useEffect(() => { load(); }, []);

  async function onAdd(e) {
    e.preventDefault();
    setErr(""); setBusy(true);
    try {
      await addChild(name.trim(), birthYear ? Number(birthYear) : null);
      setName(""); setBirthYear("");
      await load();
    } catch (e2) { setErr(e2.message); }
    finally { setBusy(false); }
  }

  return (
    <Shell>
      <h2>Children</h2>
      {loading ? <p>Loading…</p> : children.length === 0 ? (
        <p style={{ color: "#555" }}>No children yet — add one below to get started.</p>
      ) : (
        children.map((c) => <ChildCard key={c.id} child={c} />)
      )}

      <form onSubmit={onAdd} style={{ marginTop: 24, display: "flex", gap: 8, flexWrap: "wrap" }}>
        <input placeholder="Child name" value={name}
          onChange={(e) => setName(e.target.value)} required />
        <input placeholder="Birth year (optional)" value={birthYear} type="number"
          min="1990" max={new Date().getFullYear()}
          onChange={(e) => setBirthYear(e.target.value)} style={{ width: 160 }} />
        <button disabled={busy || !name.trim()}>{busy ? "Adding…" : "Add child"}</button>
      </form>
      {err && <p style={{ color: "#b00" }}>{err}</p>}

      <p style={{ color: "#888", marginTop: 40, fontSize: 13 }}>
        Transparency: everything shown here is also disclosed to your child on their device.
      </p>
    </Shell>
  );
}

function ChildCard({ child }) {
  const devices = child.devices ?? [];
  return (
    <div style={card}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <h3 style={{ margin: 0 }}>
          <Link to={`/child/${child.id}`}>{child.name}</Link>
          {child.birth_year && <span style={{ color: "#999", fontWeight: 400 }}> · {child.birth_year}</span>}
        </h3>
        <Link to={`/child/${child.id}/enroll`}>+ Enroll device</Link>
      </div>
      {devices.length === 0
        ? <p style={{ color: "#888", fontSize: 14 }}>No device enrolled yet.</p>
        : devices.map((d) => <DeviceRow key={d.id} d={d} />)}
    </div>
  );
}

function DeviceRow({ d }) {
  const silent = d.last_seen_at
    ? (Date.now() - new Date(d.last_seen_at).getTime()) > 15 * 60 * 1000
    : true;
  return (
    <div style={{ marginTop: 8, fontSize: 14 }}>
      <strong>{d.label}</strong> —{" "}
      {d.revoked_at ? "Unlinked"
        : silent ? <span style={{ color: "#b00" }}>⚠ Not checked in recently</span>
        : <span style={{ color: "#070" }}>● Active</span>}
      {d.battery_pct != null && <> · 🔋 {d.battery_pct}%</>}
      {d.foreground_app && <> · {d.foreground_app}</>}
    </div>
  );
}

export function Shell({ children }) {
  return (
    <div style={{ maxWidth: 720, margin: "40px auto", fontFamily: "system-ui", padding: "0 16px" }}>
      <header style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <Link to="/" style={{ textDecoration: "none", color: "inherit" }}><h1>FamilyGuard</h1></Link>
        <button onClick={() => supabase.auth.signOut()}>Sign out</button>
      </header>
      {children}
    </div>
  );
}

const card = { padding: 16, border: "1px solid #eee", borderRadius: 12, margin: "12px 0" };
