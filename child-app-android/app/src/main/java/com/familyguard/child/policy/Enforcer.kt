package com.familyguard.child.policy

import android.content.Context
import android.content.Intent
import com.familyguard.child.R
import com.familyguard.child.ui.BlockActivity
import java.util.Calendar

/**
 * The local decision-maker, run periodically by GuardForegroundService. It reads
 * the current foreground app + today's usage, consults the PolicyEngine, and if
 * the app should not be running, launches the friendly BlockActivity over it.
 *
 * Offline-first: everything here uses the cached policy; no network needed.
 * Emergency dialer + this app are never blocked (enforced in PolicyEngine).
 */
class Enforcer(private val context: Context) {

    private val usage = UsageTracker(context)
    private val policyStore = PolicyStore(context)

    fun enforceOnce() {
        val policy = PolicyMapper.fromJson(policyStore.load())
        val current = usage.currentForegroundApp() ?: return
        if (current in PolicyEngine.ALWAYS_ALLOWED) return

        val engine = PolicyEngine(policy)

        // Bedtime: block everything (except always-allowed) during the window.
        if (engine.isWithinBedtime(nowLocalMinutes())) {
            block(current, R.string.blocked_bedtime)
            return
        }

        when (val d = engine.decideForForegroundApp(current, usage.totalUsageTodaySeconds())) {
            is PolicyEngine.Decision.Allow -> { /* ok */ }
            is PolicyEngine.Decision.TimeLimited -> { /* within limit; a future tick will block */ }
            is PolicyEngine.Decision.Block -> block(current, d.reasonResId)
        }
    }

    private fun block(blockedPackage: String, reasonResId: Int) {
        val intent = Intent(context, BlockActivity::class.java)
            .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TOP)
            .putExtra(BlockActivity.EXTRA_REASON_RES, reasonResId)
            .putExtra(BlockActivity.EXTRA_PACKAGE, blockedPackage)
        context.startActivity(intent)
    }

    private fun nowLocalMinutes(): Int {
        val c = Calendar.getInstance()
        return c.get(Calendar.HOUR_OF_DAY) * 60 + c.get(Calendar.MINUTE)
    }
}
