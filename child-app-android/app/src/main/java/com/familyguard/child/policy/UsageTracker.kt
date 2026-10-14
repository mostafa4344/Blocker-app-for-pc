package com.familyguard.child.policy

import android.app.usage.UsageStatsManager
import android.content.Context
import java.util.Calendar

/**
 * Reads foreground-app info and per-app usage from UsageStatsManager.
 * Requires the user to grant "Usage access" (PACKAGE_USAGE_STATS is a special
 * access toggled in Settings; see PermissionsActivity).
 *
 * Only aggregate foreground time per package is read — never message/notification
 * content. This is the data the parent is told, up front, is collected.
 */
class UsageTracker(context: Context) {

    private val usm = context.getSystemService(Context.USAGE_STATS_SERVICE) as UsageStatsManager

    /** The package currently in the foreground, or null if unknown. */
    fun currentForegroundApp(): String? {
        val now = System.currentTimeMillis()
        val events = usm.queryEvents(now - 10_000, now)
        val e = android.app.usage.UsageEvents.Event()
        var last: String? = null
        while (events.hasNextEvent()) {
            events.getNextEvent(e)
            if (e.eventType == android.app.usage.UsageEvents.Event.MOVE_TO_FOREGROUND ||
                e.eventType == android.app.usage.UsageEvents.Event.ACTIVITY_RESUMED
            ) {
                last = e.packageName
            }
        }
        return last
    }

    /** Foreground seconds per package since local midnight today. */
    fun usageTodaySeconds(): Map<String, Int> {
        val start = startOfTodayMillis()
        val now = System.currentTimeMillis()
        val stats = usm.queryUsageStats(UsageStatsManager.INTERVAL_DAILY, start, now) ?: return emptyMap()
        return stats
            .filter { it.totalTimeInForeground > 0 }
            .associate { it.packageName to (it.totalTimeInForeground / 1000).toInt() }
    }

    /** Total foreground seconds across all apps today (for the daily limit). */
    fun totalUsageTodaySeconds(): Int = usageTodaySeconds().values.sum()

    private fun startOfTodayMillis(): Long {
        val c = Calendar.getInstance()
        c.set(Calendar.HOUR_OF_DAY, 0); c.set(Calendar.MINUTE, 0)
        c.set(Calendar.SECOND, 0); c.set(Calendar.MILLISECOND, 0)
        return c.timeInMillis
    }
}
