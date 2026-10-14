package com.familyguard.child.policy

/**
 * Local, offline-first policy enforcement. Holds the last-known policy and makes
 * real-time decisions without the backend in the loop (required for scale and
 * for working with no network). The backend only configures; the device decides.
 *
 * SKELETON: the decision shapes are defined; the enforcement wiring (detecting
 * the foreground app, drawing a block screen, tracking daily usage) is TODO for
 * phase 1/2. See docs/04-FEATURES.md.
 *
 * Compliance guardrails encoded here:
 *  - The emergency dialer is NEVER blocked.
 *  - FamilyGuard itself is NEVER blocked (the child must always reach it).
 */
class PolicyEngine(private val policy: Policy) {

    /** Decision for a package the child is trying to use right now. */
    sealed interface Decision {
        data object Allow : Decision
        data class Block(val reasonResId: Int) : Decision        // show friendly block screen
        data class TimeLimited(val remainingSeconds: Int) : Decision
    }

    fun decideForForegroundApp(pkg: String, usedSecondsToday: Int): Decision {
        if (pkg in ALWAYS_ALLOWED) return Decision.Allow

        val rule = policy.appRules[pkg]
        when (rule?.action) {
            AppAction.BLOCK -> return Decision.Block(/*R.string.blocked_by_guardian*/ 0)
            AppAction.ALLOW, null -> { /* fall through to limits */ }
        }

        policy.dailyLimitMin?.let { limit ->
            val remaining = limit * 60 - usedSecondsToday
            if (remaining <= 0) return Decision.Block(/*R.string.daily_limit_reached*/ 0)
            return Decision.TimeLimited(remaining)
        }
        return Decision.Allow
    }

    fun isWithinBedtime(nowLocalMinutes: Int): Boolean {
        val start = policy.bedtimeStartMin ?: return false
        val end = policy.bedtimeEndMin ?: return false
        return if (start <= end) nowLocalMinutes in start until end
        else nowLocalMinutes >= start || nowLocalMinutes < end // wraps midnight
    }

    companion object {
        /** Never blocked, regardless of policy. */
        val ALWAYS_ALLOWED = setOf(
            "com.android.dialer",            // emergency calls
            "com.android.emergency",
            "com.familyguard.child",         // this app
        )
    }
}

enum class AppAction { ALLOW, BLOCK }

data class AppRule(val action: AppAction, val limitMin: Int? = null)

/** Mirror of the backend `policies` row the device cares about. */
data class Policy(
    val version: Long,
    val dailyLimitMin: Int?,
    val bedtimeStartMin: Int?,     // minutes from local midnight
    val bedtimeEndMin: Int?,
    val locationEnabled: Boolean,
    val webFilterLevel: String,    // off | moderate | strict
    val appRules: Map<String, AppRule>,
)
