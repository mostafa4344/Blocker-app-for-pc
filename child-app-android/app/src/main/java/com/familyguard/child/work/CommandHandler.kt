package com.familyguard.child.work

import android.content.Context
import android.content.Intent
import android.os.Build
import com.familyguard.child.net.SecureStore
import com.familyguard.child.service.GuardForegroundService
import org.json.JSONArray

/**
 * Applies parent->device commands pulled by SyncWorker. Returns the list of
 * command ids that were handled, so the worker can ack them (preventing re-fire).
 */
object CommandHandler {

    fun handle(context: Context, commands: JSONArray): List<String> {
        val handled = ArrayList<String>()
        for (i in 0 until commands.length()) {
            val c = commands.optJSONObject(i) ?: continue
            val id = c.optString("id").ifBlank { null } ?: continue
            when (c.optString("kind")) {
                "lock_now", "focus_now" -> lockNow(context)
                "unlink" -> unlink(context)
                "sync", "policy_updated", "locate_now" -> { /* handled by the sync itself */ }
            }
            handled.add(id)
        }
        return handled
    }

    private fun lockNow(context: Context) {
        val svc = Intent(context, GuardForegroundService::class.java)
            .setAction(GuardForegroundService.ACTION_LOCK_NOW)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) context.startForegroundService(svc)
        else context.startService(svc)
    }

    private fun unlink(context: Context) {
        // Parent revoked this device. Stop managing: clear the credential and stop
        // the service. (The persistent notification disappears because management ended.)
        SecureStore(context).clear()
        context.stopService(Intent(context, GuardForegroundService::class.java))
    }
}
