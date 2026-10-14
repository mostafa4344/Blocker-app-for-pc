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
import org.json.JSONObject
import java.util.concurrent.TimeUnit

/**
 * Periodic sync (fallback to FCM wake). Pulls the latest policy + commands and
 * pushes status/usage. Offline-first: if the network is down, the cached policy
 * from PolicyStore keeps being enforced by the service; this worker just refreshes.
 */
class SyncWorker(ctx: Context, params: WorkerParameters) : CoroutineWorker(ctx, params) {

    override suspend fun doWork(): Result {
        val store = SecureStore(applicationContext)
        if (!store.isEnrolled) return Result.success()

        val base = store.apiBaseUrl ?: return Result.success()
        val api = Api(base, store)
        val policyStore = PolicyStore(applicationContext)

        val payload = JSONObject()
            .put("known_policy_version", policyStore.version)
            .put("status", JSONObject().put("battery_pct", Battery.level(applicationContext)))
        // TODO(phase2): attach usage batch from UsageStatsManager.

        return api.sync(payload).fold(
            onSuccess = { body ->
                body.optJSONObject("policy")?.let { policyStore.save(it) }
                // TODO: handle body.getJSONArray("commands") (lock_now, unlink, ...)
                Result.success()
            },
            onFailure = { if (it.message == "revoked") Result.success() else Result.retry() },
        )
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
