package com.familyguard.child.work

// NOTE: Enable Firebase (google-services.json + plugin + firebase-messaging dep)
// before using this. The import below resolves once the dependency is added.
//
// import com.google.firebase.messaging.FirebaseMessagingService
// import com.google.firebase.messaging.RemoteMessage
//
// A data-only FCM message ("type":"sync") is just a low-latency nudge to pull
// authoritative state. It carries nothing sensitive; the device always re-reads
// policy/commands from device-sync itself.

/*
class FcmService : FirebaseMessagingService() {
    override fun onMessageReceived(message: RemoteMessage) {
        when (message.data["type"]) {
            "sync" -> SyncWorker.syncNow(applicationContext)
        }
    }
    override fun onNewToken(token: String) {
        // TODO(phase1): upload the new FCM token via device-sync so push-command can reach us.
    }
}
*/
