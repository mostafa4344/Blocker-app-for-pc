package com.familyguard.child.policy

import android.Manifest
import android.content.Context
import android.content.pm.PackageManager
import android.location.Location
import android.location.LocationManager
import androidx.core.content.ContextCompat
import org.json.JSONObject

/**
 * Reads a single coarse location point, and ONLY when the parent has enabled
 * location sharing (checked by the caller) and the runtime permission is granted.
 * Uses the platform LocationManager (no Google Play Services dependency).
 *
 * This is "family location sharing": occasional points, not continuous tracking.
 * The child is told, via the persistent notification + disclosure, that location
 * may be shared. See docs/03-PLAY-STORE-COMPLIANCE.md and docs/07-PRIVACY-POLICY.md.
 */
class LocationReporter(private val context: Context) {

    fun hasPermission(): Boolean =
        ContextCompat.checkSelfPermission(context, Manifest.permission.ACCESS_FINE_LOCATION) ==
            PackageManager.PERMISSION_GRANTED ||
        ContextCompat.checkSelfPermission(context, Manifest.permission.ACCESS_COARSE_LOCATION) ==
            PackageManager.PERMISSION_GRANTED

    /** Best available last-known point as a JSON object, or null if unavailable. */
    fun lastKnownPoint(): JSONObject? {
        if (!hasPermission()) return null
        val lm = context.getSystemService(Context.LOCATION_SERVICE) as LocationManager
        val best = bestLastKnown(lm) ?: return null
        return JSONObject()
            .put("lat", best.latitude)
            .put("lon", best.longitude)
            .put("accuracy_m", if (best.hasAccuracy()) best.accuracy.toDouble() else JSONObject.NULL)
            .put("occurred_at", java.time.Instant.ofEpochMilli(best.time).toString())
    }

    private fun bestLastKnown(lm: LocationManager): Location? {
        val providers = listOf(LocationManager.NETWORK_PROVIDER, LocationManager.GPS_PROVIDER)
        var best: Location? = null
        for (p in providers) {
            val loc = try {
                if (lm.isProviderEnabled(p)) lm.getLastKnownLocation(p) else null
            } catch (_: SecurityException) { null }
            if (loc != null && (best == null || loc.time > best!!.time)) best = loc
        }
        return best
    }
}
