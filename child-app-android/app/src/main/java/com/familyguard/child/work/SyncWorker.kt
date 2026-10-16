package com.familyguard.child.work

import android.content.Context
import androidx.work.CoroutineWorker
import androidx.work.ExistingPeriodicWorkPolicy
import androidx.work.PeriodicWorkRequestBuilder
import androidx.work.WorkManager
import androidx.work.WorkerParameters
import com.familyguard.child.net.Api
import com.familyguard.child.net.SecureStore
import com.familyguard.child.policy.PolicyStore
import com.familyguard.child.policy.UsageTracker
import org.json.JSONArray
import org.json.JSONObject
import java.util.concurrent.TimeUnit

/**
 * Periodic sync (FCM wake triggers an immediate run). Pulls the latest policy +
 * commands, pushes status/usage, applies commands, then acks them.
 *
 * Offline-first: if the network is down, the cached policy keeps being enforced
 * by the service; this worker only refreshes config and telemetry.
 */
class SyncWorker(ctx: Context, params: WorkerParameters) : CoroutineWorker(ctx, params) {

    override suspend fun doWork(): Result {
        val store = SecureStore(applicationContext)
        if (!store.isEnrolled) return Result.success()
        val base = store.apiBaseUrl ?: return Result.success()
        val api = Api(base, store)
        val policyStore = PolicyStore(applicationContext)
        val usage = UsageTracker(applicationContext)

        val status = JSONObject()
            .put("battery_pct", Battery.level(applicationContext))
            .put("foreground_app", usage.currentForegroundApp())
        store.pushToken?.let { status.put("push_token", it) }

        val payload = JSONObject()
            .put("known_policy_version", policyStore.version)
            .put("status", status)
            .put("usage", usageBatch(usage))

        // Location only when the active policy enables it (and permission granted).
        val policy = com.familyguard.child.policy.PolicyMapper.fromJson(policyStore.load())
        if (policy.locationEnabled) {
            com.familyguard.child.policy.LocationReporter(applicationContext).lastKnownPoint()?.let {
                payload.put("locations", JSONArray().put(it))
            }
        }

        return api.sync(payload).fold(
            onSuccess = { body ->
                body.optJSONObject("policy")?.let { policyStore.save(it) }
                val commands = body.optJSONArray("commands") ?: JSONArray()
                val handled = CommandHandler.handle(applicationContext, commands)
                if (handled.isNotEmpty() && SecureStore(applicationContext).isEnrolled) {
                    // Ack handled commands so they are not returned/re-fired next poll.
                    val ackPayload = JSONObject().put("ack_command_ids", JSONArray(handled))
                    api.sync(ackPayload)
                }
                Result.success()
            },
            onFailure = { if (it.message == "revoked") Result.success() else Result.retry() },
        )
    }

    /**
     * Today's per-app cumulative foreground seconds. The backend UPSERTs these
     * onto the (device, package, day) row, so sending cumulative totals each
     * sync is correct — repeated syncs overwrite, never double-count.
     */
    private fun usageBatch(usage: UsageTracker): JSONArray {
        val arr = JSONArray()
        val nowIso = java.time.Instant.now().toString()
        for ((pkg, secs) in usage.usageTodaySeconds()) {
            arr.put(JSONObject()
                .put("package_name", pkg)
                .put("foreground_s", secs)
                .put("occurred_at", nowIso))
        }
        return arr
    }

    companion object {
        private const val NAME = "familyguard_sync"
        fun schedule(ctx: Context) {
            val req = PeriodicWorkRequestBuilder<SyncWorker>(15, TimeUnit.MINUTES).build()
            WorkManager.getInstance(ctx)
                .enqueueUniquePeriodicWork(NAME, ExistingPeriodicWorkPolicy.KEEP, req)
        }
        /** Trigger an immediate one-off sync (used by the FCM wake). */
        fun syncNow(ctx: Context) {
            val req = androidx.work.OneTimeWorkRequestBuilder<SyncWorker>().build()
            WorkManager.getInstance(ctx).enqueue(req)
        }
    }
}

private object Battery {
    fun level(ctx: Context): Int {
        val bm = ctx.getSystemService(Context.BATTERY_SERVICE) as android.os.BatteryManager
        return bm.getIntProperty(android.os.BatteryManager.BATTERY_PROPERTY_CAPACITY)
    }
}
