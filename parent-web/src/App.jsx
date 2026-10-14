import { useEffect, useState } from "react";
import { Routes, Route, Navigate } from "react-router-dom";
import { supabase } from "./lib/supabase.js";
import Login from "./pages/Login.jsx";
import Dashboard from "./pages/Dashboard.jsx";
import ChildDetail from "./pages/ChildDetail.jsx";
import EnrollDevice from "./pages/EnrollDevice.jsx";

/**
 * Auth gate + routes. RLS does the real per-family isolation on the server;
 * this is just the UI gate.
 */
export default function App() {
  const [session, setSession] = useState(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    supabase.auth.getSession().then(({ data }) => {
      setSession(data.session);
      setLoading(false);
    });
    const { data: sub } = supabase.auth.onAuthStateChange((_e, s) => setSession(s));
    return () => sub.subscription.unsubscribe();
  }, []);

  if (loading) return <div style={{ padding: 24 }}>Loading…</div>;
  if (!session) {
    return (
      <Routes>
        <Route path="*" element={<Login />} />
      </Routes>
    );
  }

  return (
    <Routes>
      <Route path="/" element={<Dashboard />} />
      <Route path="/child/:childId" element={<ChildDetail />} />
      <Route path="/child/:childId/enroll" element={<EnrollDevice />} />
      <Route path="*" element={<Navigate to="/" />} />
    </Routes>
  );
}
