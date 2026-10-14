package com.familyguard.child.net

import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import org.json.JSONObject
import java.util.concurrent.TimeUnit

/**
 * Thin client for the device-facing Edge Functions. All device calls (except
 * enrollment) send the per-device JWT in the `x-device-jwt` header. The device
 * id is never sent in the body — the server derives it from the JWT.
 */
class Api(private val baseUrl: String, private val store: SecureStore) {

    private val http = OkHttpClient.Builder()
        .callTimeout(30, TimeUnit.SECONDS)
        .build()

    private val jsonType = "application/json".toMediaType()

    /** Enrollment: exchange a raw enrollment token for a device JWT. */
    fun enroll(rawToken: String, device: JSONObject): Result<Unit> = runCatching {
        val body = JSONObject().put("token", rawToken).put("device", device)
        val req = Request.Builder()
            .url("$baseUrl/enroll-device")
            .post(body.toString().toRequestBody(jsonType))
            .build()
        http.newCall(req).execute().use { res ->
            val text = res.body?.string().orEmpty()
            if (!res.isSuccessful) error("enroll failed ${res.code}: $text")
            val j = JSONObject(text)
            store.deviceJwt = j.getString("device_jwt")
            store.deviceId = j.getString("device_id")
            store.apiBaseUrl = baseUrl
        }
    }

    /** Main sync: push status/usage, pull policy + commands. Returns the JSON body. */
    fun sync(payload: JSONObject): Result<JSONObject> = runCatching {
        val jwt = store.deviceJwt ?: error("not enrolled")
        val req = Request.Builder()
            .url("${store.apiBaseUrl ?: baseUrl}/device-sync")
            .header("x-device-jwt", jwt)
            .post(payload.toString().toRequestBody(jsonType))
            .build()
        http.newCall(req).execute().use { res ->
            val text = res.body?.string().orEmpty()
            if (res.code == 403) { // revoked -> stand down
                store.clear()
                error("revoked")
            }
            if (!res.isSuccessful) error("sync failed ${res.code}: $text")
            JSONObject(text)
        }
    }

    /** Lightweight heartbeat. */
    fun heartbeat(payload: JSONObject): Result<Unit> = runCatching {
        val jwt = store.deviceJwt ?: error("not enrolled")
        val req = Request.Builder()
            .url("${store.apiBaseUrl ?: baseUrl}/device-heartbeat")
            .header("x-device-jwt", jwt)
            .post(payload.toString().toRequestBody(jsonType))
            .build()
        http.newCall(req).execute().use { res ->
            if (!res.isSuccessful) error("heartbeat failed ${res.code}")
        }
    }
}
