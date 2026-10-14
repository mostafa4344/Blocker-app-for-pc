package com.familyguard.child

import android.app.Application
import android.app.NotificationChannel
import android.app.NotificationManager
import android.content.Context
import android.os.Build

/**
 * Application entry point. Creates the notification channel used by the
 * mandatory persistent status notification.
 *
 * COMPLIANCE: the status channel is IMPORTANCE_LOW (quiet, but always visible)
 * and the notification itself is ongoing/non-dismissible. The child must always
 * be able to see that the device is managed. See docs/03-PLAY-STORE-COMPLIANCE.md.
 */
class FamilyGuardApp : Application() {

    override fun onCreate() {
        super.onCreate()
        createStatusChannel()
    }

    private fun createStatusChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                STATUS_CHANNEL_ID,
                getString(R.string.status_channel_name),
                NotificationManager.IMPORTANCE_LOW,
            ).apply {
                description = getString(R.string.status_channel_desc)
                setShowBadge(false)
            }
            getSystemService(NotificationManager::class.java).createNotificationChannel(channel)
        }
    }

    companion object {
        const val STATUS_CHANNEL_ID = "familyguard_status"
    }
}
