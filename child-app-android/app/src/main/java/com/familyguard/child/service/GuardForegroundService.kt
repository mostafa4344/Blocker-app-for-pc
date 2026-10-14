package com.familyguard.child.service

import android.app.Notification
import android.app.PendingIntent
import android.content.Intent
import android.app.Service
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.IBinder
import androidx.core.app.NotificationCompat
import com.familyguard.child.FamilyGuardApp
import com.familyguard.child.R
import com.familyguard.child.ui.TransparencyActivity

/**
 * The always-on foreground service whose only guaranteed job is to display the
 * mandatory persistent status notification.
 *
 * COMPLIANCE (C1): This notification is ongoing and non-dismissible. Its text
 * makes clear the device is managed and that a parent/guardian can see activity.
 * Tapping it opens the child-facing Transparency screen ("what my guardian can
 * see"). Removing this notification is NOT a feature and must never be added.
 *
 * The PolicyEngine and sync run alongside this, but the notification is the
 * compliance anchor and is created first, within the 5s foreground window.
 */
class GuardForegroundService : Service() {

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        startForegroundCompliant()
        // TODO(phase1): start PolicyEngine enforcement loop + schedule SyncWorker here.
        return START_STICKY // restart if killed; the status must persist
    }

    private fun startForegroundCompliant() {
        val tapIntent = PendingIntent.getActivity(
            this, 0,
            Intent(this, TransparencyActivity::class.java),
            PendingIntent.FLAG_IMMUTABLE,
        )

        val notification: Notification =
            NotificationCompat.Builder(this, FamilyGuardApp.STATUS_CHANNEL_ID)
                .setSmallIcon(R.drawable.ic_shield)
                .setContentTitle(getString(R.string.status_notif_title))
                .setContentText(getString(R.string.status_notif_text))
                .setStyle(NotificationCompat.BigTextStyle().bigText(getString(R.string.status_notif_text)))
                .setOngoing(true)                 // non-dismissible (compliance)
                .setShowWhen(false)
                .setContentIntent(tapIntent)
                .setCategory(NotificationCompat.CATEGORY_SERVICE)
                .build()

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
            startForeground(NOTIF_ID, notification, ServiceInfo.FOREGROUND_SERVICE_TYPE_SPECIAL_USE)
        } else {
            startForeground(NOTIF_ID, notification)
        }
    }

    override fun onBind(intent: Intent?): IBinder? = null

    companion object {
        private const val NOTIF_ID = 1001
    }
}
