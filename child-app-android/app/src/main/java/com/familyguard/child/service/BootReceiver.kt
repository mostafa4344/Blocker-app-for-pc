package com.familyguard.child.service

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.os.Build
import com.familyguard.child.net.SecureStore
import com.familyguard.child.work.SyncWorker

/** Restart the persistent status service + reschedule sync after a reboot. */
class BootReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        if (intent.action != Intent.ACTION_BOOT_COMPLETED) return
        if (!SecureStore(context).isEnrolled) return
        val svc = Intent(context, GuardForegroundService::class.java)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) context.startForegroundService(svc)
        else context.startService(svc)
        SyncWorker.schedule(context)
    }
}
