// Shared CORS headers for Edge Functions.
// Lock `Access-Control-Allow-Origin` to your real web origin(s) before production.
export const corsHeaders = {
  "Access-Control-Allow-Origin": "*", // TODO: set to https://app.yourdomain.com in prod
  "Access-Control-Allow-Headers":
    "authorization, x-client-info, apikey, content-type, x-device-jwt",
  "Access-Control-Allow-Methods": "POST, OPTIONS",
};

export function handleOptions(): Response {
  return new Response("ok", { headers: corsHeaders });
}

export function json(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { ...corsHeaders, "Content-Type": "application/json" },
  });
}
