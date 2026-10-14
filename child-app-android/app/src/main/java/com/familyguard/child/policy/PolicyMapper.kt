package com.familyguard.child.policy

import org.json.JSONObject

/**
 * Converts the policy JSON stored by PolicyStore (the shape device-sync returns)
 * into the typed Policy the PolicyEngine uses. Tolerant of missing fields.
 */
object PolicyMapper {

    fun fromJson(j: JSONObject?): Policy {
        if (j == null) return EMPTY
        val rules = j.optJSONObject("rules") ?: JSONObject()
        val apps = HashMap<String, AppRule>()
        rules.optJSONArray("apps")?.let { arr ->
            for (i in 0 until arr.length()) {
                val a = arr.optJSONObject(i) ?: continue
                val pkg = a.optString("pkg")
                if (pkg.isBlank()) continue
                val action = if (a.optString("action") == "block") AppAction.BLOCK else AppAction.ALLOW
                val limit = if (a.has("limitMin") && !a.isNull("limitMin")) a.optInt("limitMin") else null
                apps[pkg] = AppRule(action, limit)
            }
        }
        return Policy(
            version = j.optLong("version", -1),
            dailyLimitMin = if (j.isNull("daily_limit_min")) null else j.optInt("daily_limit_min"),
            bedtimeStartMin = parseTimeToMinutes(j.optString("bedtime_start")),
            bedtimeEndMin = parseTimeToMinutes(j.optString("bedtime_end")),
            locationEnabled = j.optBoolean("location_enabled", false),
            webFilterLevel = j.optString("web_filter_level", "moderate"),
            appRules = apps,
        )
    }

    /** "21:00" or "21:00:00" -> minutes from midnight; blank -> null. */
    private fun parseTimeToMinutes(s: String?): Int? {
        if (s.isNullOrBlank()) return null
        val parts = s.split(":")
        if (parts.size < 2) return null
        val h = parts[0].toIntOrNull() ?: return null
        val m = parts[1].toIntOrNull() ?: return null
        return h * 60 + m
    }

    private val EMPTY = Policy(
        version = -1, dailyLimitMin = null, bedtimeStartMin = null, bedtimeEndMin = null,
        locationEnabled = false, webFilterLevel = "moderate", appRules = emptyMap(),
    )
}
