package com.familyguard.child.net

import android.content.Context
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKey

/**
 * Keystore-backed encrypted storage for the per-device JWT and device id.
 * The device JWT is the only credential this app holds; keep it here, never in
 * plain SharedPreferences or logs. See docs/02-SECURITY.md.
 */
class SecureStore(context: Context) {

    private val prefs = EncryptedSharedPreferences.create(
        context,
        "familyguard_secure",
        MasterKey.Builder(context).setKeyScheme(MasterKey.KeyScheme.AES256_GCM).build(),
        EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
        EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM,
    )

    var deviceJwt: String?
        get() = prefs.getString(KEY_JWT, null)
        set(v) = prefs.edit().putString(KEY_JWT, v).apply()

    var deviceId: String?
        get() = prefs.getString(KEY_DEVICE_ID, null)
        set(v) = prefs.edit().putString(KEY_DEVICE_ID, v).apply()

    var apiBaseUrl: String?
        get() = prefs.getString(KEY_API, null)
        set(v) = prefs.edit().putString(KEY_API, v).apply()

    val isEnrolled: Boolean get() = deviceJwt != null

    fun clear() = prefs.edit().clear().apply()

    companion object {
        private const val KEY_JWT = "device_jwt"
        private const val KEY_DEVICE_ID = "device_id"
        private const val KEY_API = "api_base_url"
    }
}
