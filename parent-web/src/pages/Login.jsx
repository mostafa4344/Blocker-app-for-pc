import { useState } from "react";
import { supabase } from "../lib/supabase.js";

export default function Login() {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [mode, setMode] = useState("signin"); // signin | signup
  const [msg, setMsg] = useState("");

  async function submit(e) {
    e.preventDefault();
    setMsg("");
    const fn = mode === "signin"
      ? supabase.auth.signInWithPassword({ email, password })
      : supabase.auth.signUp({ email, password });
    const { error } = await fn;
    if (error) setMsg(error.message);
    else if (mode === "signup") setMsg("Check your email to confirm, then sign in.");
  }

  return (
    <div style={{ maxWidth: 360, margin: "80px auto", fontFamily: "system-ui" }}>
      <h1>FamilyGuard</h1>
      <p style={{ color: "#555" }}>Parent sign {mode === "signin" ? "in" : "up"}</p>
      <form onSubmit={submit} style={{ display: "grid", gap: 10 }}>
        <input type="email" placeholder="Email" value={email}
          onChange={(e) => setEmail(e.target.value)} required />
        <input type="password" placeholder="Password" value={password}
          onChange={(e) => setPassword(e.target.value)} required minLength={8} />
        <button type="submit">{mode === "signin" ? "Sign in" : "Create account"}</button>
      </form>
      <button style={{ marginTop: 12 }} onClick={() => setMode(mode === "signin" ? "signup" : "signin")}>
        {mode === "signin" ? "Need an account? Sign up" : "Have an account? Sign in"}
      </button>
      {msg && <p style={{ color: "#b00" }}>{msg}</p>}
    </div>
  );
}
