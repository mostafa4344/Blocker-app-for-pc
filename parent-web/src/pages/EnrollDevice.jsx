import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import { QRCodeSVG } from "qrcode.react";
import { supabase } from "../lib/supabase.js";
import { callFunction } from "../lib/functions.js";
import { FUNCTIONS_URL } from "../lib/supabase.js";
import { Shell } from "./Dashboard.jsx";

/**
 * Shows a short-lived enrollment QR. The parent installs the FamilyGuard app on
 * the child phone and scans this. The QR encodes a JSON payload with the raw
 * enrollment token + the functions base URL the device should call.
 *
 * The raw token is shown once and expires quickly (single-use). Re-generate if
 * it expires before the child device scans it.
 */
export default function EnrollDevice() {
  const { childId } = useParams();
  const [child, setChild] = useState(null);
  const [payload, setPayload] = useState("");
  const [expiresAt, setExpiresAt] = useState(null);
  const [err, setErr] = useState("");
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    supabase.from("children").select("id,name").eq("id", childId).maybeSingle()
      .then(({ data }) => setChild(data));
  }, [childId]);

  async function generate() {
    setErr(""); setBusy(true); setPayload("");
    try {
      const { token, expires_at } = await callFunction("create-enrollment-token", { child_id: childId });
      // The device parses this JSON from the QR.
      setPayload(JSON.stringify({ v: 1, token, api: FUNCTIONS_URL }));
      setExpiresAt(expires_at);
    } catch (e) { setErr(e.message); }
    finally { setBusy(false); }
  }

  return (
    <Shell>
      <h2>Enroll a device{child ? ` for ${child.name}` : ""}</h2>
      <ol style={{ color: "#444", lineHeight: 1.6 }}>
        <li>On the child's Android phone, install the <strong>FamilyGuard</strong> app.</li>
        <li>Open it — it shows the disclosure screen, then a QR scanner.</li>
        <li>Tap <em>Generate code</em> below and scan it with the phone.</li>
        <li>The phone enrolls, shows its permanent status notification, and appears on your dashboard.</li>
      </ol>

      <button onClick={generate} disabled={busy} style={{ padding: "8px 16px" }}>
        {busy ? "Generating…" : payload ? "Regenerate code" : "Generate code"}
      </button>
      {err && <p style={{ color: "#b00" }}>{err}</p>}

      {payload && (
        <div style={{ marginTop: 20, textAlign: "center" }}>
          <div style={{ display: "inline-block", padding: 16, background: "#fff", border: "1px solid #eee", borderRadius: 12 }}>
            <QRCodeSVG value={payload} size={220} />
          </div>
          <p style={{ color: "#888", fontSize: 13 }}>
            Expires {expiresAt ? new Date(expiresAt).toLocaleTimeString() : "soon"}. Single use.
          </p>
        </div>
      )}
    </Shell>
  );
}
