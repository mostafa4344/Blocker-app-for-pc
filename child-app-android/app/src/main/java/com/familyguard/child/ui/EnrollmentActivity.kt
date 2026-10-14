package com.familyguard.child.ui

import android.app.Activity
import android.app.admin.DevicePolicyManager
import android.content.Context
import android.content.Intent
import android.os.Build
import android.os.Bundle
import android.widget.Button
import android.widget.TextView
import android.widget.Toast
import com.familyguard.child.R
import com.familyguard.child.admin.GuardDeviceAdminReceiver
import com.familyguard.child.net.Api
import com.familyguard.child.net.SecureStore
import com.familyguard.child.service.GuardForegroundService
import com.google.zxing.integration.android.IntentIntegrator
import org.json.JSONObject
import kotlin.concurrent.thread

/**
 * Launcher activity + enrollment flow (Compliance: disclosure BEFORE binding).
 *
 *   not enrolled  -> [Start setup] -> DisclosureActivity -> QR scan -> enroll
 *   enrolled      -> "Protection is active" + open Transparency screen
 *
 * The app icon / this activity is always present in the launcher (Compliance C2).
 */
class EnrollmentActivity : Activity() {

    private lateinit var store: SecureStore

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_enrollment)
        store = SecureStore(this)
        render()

        findViewById<Button>(R.id.startButton).setOnClickListener {
            // Show disclosure first; enrollment continues after consent.
            startActivityForResult(Intent(this, DisclosureActivity::class.java), REQ_DISCLOSURE)
        }
        findViewById<Button>(R.id.transparencyButton).setOnClickListener {
            startActivity(Intent(this, TransparencyActivity::class.java))
        }
    }

    private fun render() {
        val enrolled = store.isEnrolled
        findViewById<TextView>(R.id.statusText).text =
            if (enrolled) getString(R.string.enroll_active) else getString(R.string.enroll_needed)
        findViewById<Button>(R.id.startButton).visibility =
            if (enrolled) android.view.View.GONE else android.view.View.VISIBLE
        findViewById<Button>(R.id.transparencyButton).visibility =
            if (enrolled) android.view.View.VISIBLE else android.view.View.GONE
        if (enrolled) ensureServiceRunning()
    }

    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        if (requestCode == REQ_DISCLOSURE && resultCode == RESULT_OK) {
            // Consent recorded -> open the QR scanner.
            IntentIntegrator(this).setPrompt(getString(R.string.scan_prompt))
                .setBeepEnabled(false).setOrientationLocked(true).initiateScan()
            return
        }
        val scan = IntentIntegrator.parseActivityResult(requestCode, resultCode, data)
        if (scan != null && scan.contents != null) {
            handleScannedPayload(scan.contents)
            return
        }
        super.onActivityResult(requestCode, resultCode, data)
    }

    private fun handleScannedPayload(contents: String) {
        val payload = runCatching { JSONObject(contents) }.getOrNull()
        val token = payload?.optString("token")
        val api = payload?.optString("api")
        if (token.isNullOrBlank() || api.isNullOrBlank()) {
            toast(getString(R.string.scan_invalid)); return
        }
        toast(getString(R.string.enroll_working))
        thread {
            val device = JSONObject()
                .put("label", Build.MODEL ?: "Phone")
                .put("model", Build.MODEL)
                .put("os_version", "Android ${Build.VERSION.RELEASE}")
                .put("management_mode", if (isDeviceOwner()) "device_owner" else "device_admin")
            val result = Api(api, store).enroll(token, device)
            runOnUiThread {
                result.fold(
                    onSuccess = {
                        promptDeviceAdminIfNeeded()
                        ensureServiceRunning()
                        render()
                        toast(getString(R.string.enroll_done))
                        // Guide the user through the special accesses enforcement needs.
                        startActivity(Intent(this, PermissionsActivity::class.java))
                    },
                    onFailure = { toast(getString(R.string.enroll_failed)) },
                )
            }
        }
    }

    private fun isDeviceOwner(): Boolean {
        val dpm = getSystemService(Context.DEVICE_POLICY_SERVICE) as DevicePolicyManager
        return dpm.isDeviceOwnerApp(packageName)
    }

    private fun promptDeviceAdminIfNeeded() {
        val dpm = getSystemService(Context.DEVICE_POLICY_SERVICE) as DevicePolicyManager
        val admin = GuardDeviceAdminReceiver.componentName(this)
        if (!dpm.isAdminActive(admin) && !isDeviceOwner()) {
            val intent = Intent(DevicePolicyManager.ACTION_ADD_DEVICE_ADMIN)
                .putExtra(DevicePolicyManager.EXTRA_DEVICE_ADMIN, admin)
                .putExtra(DevicePolicyManager.EXTRA_ADD_EXPLANATION, getString(R.string.admin_explanation))
            startActivity(intent)
        }
    }

    private fun ensureServiceRunning() {
        val svc = Intent(this, GuardForegroundService::class.java)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) startForegroundService(svc) else startService(svc)
    }

    private fun toast(m: String) = Toast.makeText(this, m, Toast.LENGTH_SHORT).show()

    companion object { private const val REQ_DISCLOSURE = 42 }
}
