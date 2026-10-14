package com.familyguard.child.service

import android.app.Notification
import android.app.PendingIntent
import android.app.admin.DevicePolicyManager
import android.content.Context
import android.content.Intent
import android.app.Service
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.Handler
import android.os.HandlerThread
import android.os.IBinder
import androidx.core.app.NotificationCompat
import com.familyguard.child.FamilyGuardApp
import com.familyguard.child.R
import com.familyguard.child.admin.GuardDeviceAdminReceiver
import com.familyguard.child.policy.Enforcer
import com.familyguard.child.ui.TransparencyActivity
import com.familyguard.child.work.SyncWorker

/**
 * Always-on foreground service. Its guaranteed job is the mandatory persistent
 * status notification (Compliance C1). It also runs the local enforcement loop
 * (Enforcer) on a background thread and handles the "lock now" action.
 *
 * The notification is created first, inside the foreground-start window, so the
 * compliance anchor is never at risk if enforcement setup is slow.
 */
class GuardForegroundService : Service() {

    private lateinit var worker: HandlerThread
    private lateinit var handler: Handler
    private lateinit var enforcer: Enforcer
    @Volatile private var running = false

    private val tick = object : Runnable {
        override fun run() {
            runCatching { enforcer.enforceOnce() }
            if (running) handler.postDelayed(this, ENFORCE_INTERVAL_MS)
        }
    }

    override fun onCreate() {
        super.onCreate()
        enforcer = Enforcer(applicationContext)
        worker = HandlerThread("fg-enforcer").apply { start() }
        handler = Handler(worker.looper)
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        startForegroundCompliant()

        if (intent?.action == ACTION_LOCK_NOW) {
            lockDevice()
        }

        if (!running) {
            running = true
            handler.post(tick)
            SyncWorker.schedule(applicationContext)
        }
        return START_STICKY
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
                .setOngoing(true)
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

    /** Lock the screen immediately if we hold device-admin/owner. */
    private fun lockDevice() {
        val dpm = getSystemService(Context.DEVICE_POLICY_SERVICE) as DevicePolicyManager
        if (dpm.isAdminActive(GuardDeviceAdminReceiver.componentName(this)) ||
            dpm.isDeviceOwnerApp(packageName)
        ) {
            runCatching { dpm.lockNow() }
        }
    }

    override fun onDestroy() {
        running = false
        if (::worker.isInitialized) worker.quitSafely()
        super.onDestroy()
    }

    override fun onBind(intent: Intent?): IBinder? = null

    companion object {
        private const val NOTIF_ID = 1001
        private const val ENFORCE_INTERVAL_MS = 2000L
        const val ACTION_LOCK_NOW = "com.familyguard.child.LOCK_NOW"
    }
}
