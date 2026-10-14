package com.familyguard.child.admin

import android.app.admin.DeviceAdminReceiver
import android.content.ComponentName
import android.content.Context
import android.content.Intent

/**
 * Device Admin receiver — the hook that provides removal resistance.
 *
 * When the app is a Device Admin (and, ideally, Device Owner), Android blocks a
 * normal uninstall and warns before the admin is disabled. We also show a clear
 * reason to the child when disabling is requested.
 *
 * HONESTY: this is resistance, not a guarantee. Factory reset / recovery /
 * reflash can still remove the app. The backend detects removal via heartbeat
 * loss ("device went silent") rather than pretending it is impossible.
 * See docs/02-SECURITY.md and docs/03-PLAY-STORE-COMPLIANCE.md.
 */
class GuardDeviceAdminReceiver : DeviceAdminReceiver() {

    override fun onEnabled(context: Context, intent: Intent) {
        // Admin granted — ensure the persistent status service is running.
        // TODO(phase1): startForegroundService(GuardForegroundService)
    }

    override fun onDisableRequested(context: Context, intent: Intent): CharSequence {
        // Shown to the user before they can disable admin. Keep it honest + calm.
        return context.getString(com.familyguard.child.R.string.admin_disable_warning)
    }

    override fun onDisabled(context: Context, intent: Intent) {
        // Admin removed. TODO(phase1): report this state up so the parent sees it.
    }

    companion object {
        fun componentName(context: Context) =
            ComponentName(context, GuardDeviceAdminReceiver::class.java)
    }
}
