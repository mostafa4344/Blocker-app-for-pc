package com.familyguard.child.policy

import android.content.Context
import org.json.JSONObject

/**
 * Caches the last-known policy on disk so enforcement works offline. Stores the
 * raw policy JSON from device-sync plus its version for change detection.
 * (Non-secret config, so plain SharedPreferences is fine.)
 */
class PolicyStore(context: Context) {
    private val prefs = context.getSharedPreferences("familyguard_policy", Context.MODE_PRIVATE)

    val version: Long get() = prefs.getLong(KEY_VERSION, -1)

    fun save(policyJson: JSONObject) {
        prefs.edit()
            .putString(KEY_JSON, policyJson.toString())
            .putLong(KEY_VERSION, policyJson.optLong("version", version))
            .apply()
    }

    fun load(): JSONObject? =
        prefs.getString(KEY_JSON, null)?.let { runCatching { JSONObject(it) }.getOrNull() }

    companion object {
        private const val KEY_JSON = "policy_json"
        private const val KEY_VERSION = "policy_version"
    }
}
