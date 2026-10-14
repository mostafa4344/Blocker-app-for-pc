import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import { supabase } from "../lib/supabase.js";
import { callFunction } from "../lib/functions.js";
import { Shell } from "./Dashboard.jsx";

/**
 * Policy editor for one child + device commands. Saving a policy writes to the
 * `policies` row (RLS-checked); a trigger bumps the version, and we also nudge
 * the device via push-command(kind='policy_updated'). "Lock now" sends a command.
 */
export default function ChildDetail() {
  const { childId } = useParams();
  const [child, setChild] = useState(null);
  const [policy, setPolicy] = useState(null);
  const [devices, setDevices] = useState([]);
  const [blockPkg, setBlockPkg] = useState("");
  const [msg, setMsg] = useState("");
  const [err, setErr] = useState("");

  async function load() {
    const { data: c } = await supabase
      .from("children").select("id, name, birth_year").eq("id", childId).maybeSingle();
    setChild(c);
    const { data: p } = await supabase
      .from("policies").select("*").eq("child_id", childId).eq("is_active", true).maybeSingle();
    setPolicy(p);
    const { data: d } = await supabase
      .from("devices").select("id, label, last_seen_at, revoked_at").eq("child_id", childId);
    setDevices(d ?? []);
  }
  useEffect(() => { load(); }, [childId]);

  function setField(k, v) { setPolicy((p) => ({ ...p, [k]: v })); }

  function blockedApps() {
    const apps = policy?.rules?.apps ?? [];
    return apps.filter((a) => a.action === "block");
  }
  function addBlock() {
    if (!blockPkg.trim()) return;
    const apps = policy?.rules?.apps ?? [];
    if (apps.some((a) => a.pkg === blockPkg.trim())) return;
    setField("rules", { ...(policy.rules ?? {}), apps: [...apps, { pkg: blockPkg.trim(), action: "block" }] });
    setBlockPkg("");
  }
  function removeBlock(pkg) {
    const apps = (policy?.rules?.apps ?? []).filter((a) => a.pkg !== pkg);
    setField("rules", { ...(policy.rules ?? {}), apps });
  }

  async function save() {
    setErr(""); setMsg("");
    const { error } = await supabase.from("policies").update({
      daily_limit_min: policy.daily_limit_min || null,
      bedtime_start: policy.bedtime_start || null,
      bedtime_end: policy.bedtime_end || null,
      location_enabled: !!policy.location_enabled,
      web_filter_level: policy.web_filter_level || "moderate",
      rules: policy.rules ?? {},
    }).eq("id", policy.id);
    if (error) { setErr(error.message); return; }

    // Nudge each active device to sync the new policy.
    for (const d of devices.filter((x) => !x.revoked_at)) {
      try { await callFunction("push-command", { device_id: d.id, kind: "policy_updated" }); }
      catch { /* best-effort; device also polls */ }
    }
    setMsg("Saved. Devices will apply shortly.");
    load();
  }

  async function lockNow(deviceId) {
    setErr(""); setMsg("");
    try { await callFunction("push-command", { device_id: deviceId, kind: "lock_now" }); setMsg("Lock sent."); }
    catch (e) { setErr(e.message); }
  }
  async function unlink(deviceId) {
    if (!confirm("Unlink this device? It will stop being managed.")) return;
    await supabase.from("devices").update({ revoked_at: new Date().toISOString() }).eq("id", deviceId);
    load();
  }

  if (!child) return <Shell><p>Loading…</p></Shell>;
  if (!policy) return <Shell><p>No active policy found for {child.name}.</p></Shell>;

  return (
    <Shell>
      <h2>{child.name}</h2>

      <section style={box}>
        <h3>Screen time</h3>
        <label style={row}>
          Daily limit (minutes)
          <input type="number" min="0" value={policy.daily_limit_min ?? ""}
            onChange={(e) => setField("daily_limit_min", e.target.value ? Number(e.target.value) : null)} />
        </label>
        <label style={row}>
          Bedtime start
          <input type="time" value={policy.bedtime_start ?? ""}
            onChange={(e) => setField("bedtime_start", e.target.value)} />
        </label>
        <label style={row}>
          Bedtime end
          <input type="time" value={policy.bedtime_end ?? ""}
            onChange={(e) => setField("bedtime_end", e.target.value)} />
        </label>
      </section>

      <section style={box}>
        <h3>Web filtering</h3>
        <select value={policy.web_filter_level ?? "moderate"}
          onChange={(e) => setField("web_filter_level", e.target.value)}>
          <option value="off">Off</option>
          <option value="moderate">Moderate (SafeSearch)</option>
          <option value="strict">Strict</option>
        </select>
      </section>

      <section style={box}>
        <h3>Blocked apps</h3>
        {blockedApps().length === 0 && <p style={{ color: "#888" }}>None blocked.</p>}
        {blockedApps().map((a) => (
          <div key={a.pkg} style={{ display: "flex", justifyContent: "space-between" }}>
            <code>{a.pkg}</code>
            <button onClick={() => removeBlock(a.pkg)}>Remove</button>
          </div>
        ))}
        <div style={{ display: "flex", gap: 8, marginTop: 8 }}>
          <input placeholder="e.g. com.zhiliaoapp.musically (TikTok)" value={blockPkg}
            onChange={(e) => setBlockPkg(e.target.value)} style={{ flex: 1 }} />
          <button onClick={addBlock}>Add</button>
        </div>
      </section>

      <section style={box}>
        <h3>Location sharing</h3>
        <label>
          <input type="checkbox" checked={!!policy.location_enabled}
            onChange={(e) => setField("location_enabled", e.target.checked)} />
          {" "}Share this device's location (child is notified on their phone)
        </label>
      </section>

      <button onClick={save} style={{ padding: "8px 16px" }}>Save policy</button>
      {msg && <span style={{ color: "#070", marginLeft: 12 }}>{msg}</span>}
      {err && <span style={{ color: "#b00", marginLeft: 12 }}>{err}</span>}

      <section style={box}>
        <h3>Devices</h3>
        {devices.length === 0 && <p style={{ color: "#888" }}>No device enrolled.</p>}
        {devices.map((d) => (
          <div key={d.id} style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 6 }}>
            <span>{d.label} {d.revoked_at ? "(unlinked)" : ""}</span>
            {!d.revoked_at && (
              <span style={{ display: "flex", gap: 8 }}>
                <button onClick={() => lockNow(d.id)}>Lock now</button>
                <button onClick={() => unlink(d.id)}>Unlink</button>
              </span>
            )}
          </div>
        ))}
      </section>
    </Shell>
  );
}

const box = { padding: 16, border: "1px solid #eee", borderRadius: 12, margin: "12px 0" };
const row = { display: "flex", justifyContent: "space-between", alignItems: "center", margin: "6px 0" };
