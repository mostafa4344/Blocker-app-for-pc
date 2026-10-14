package com.familyguard.child.ui

import android.app.Activity
import android.content.Intent
import android.os.Bundle
import android.widget.Button
import android.widget.TextView
import com.familyguard.child.R

/**
 * Full-screen, friendly block shown over a blocked/limited app. The tone is calm
 * and non-shaming (see docs/04-FEATURES.md). "Go home" sends the child to the
 * launcher; emergency calls are never blocked (that app is in ALWAYS_ALLOWED).
 */
class BlockActivity : Activity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_block)

        val reasonRes = intent.getIntExtra(EXTRA_REASON_RES, R.string.blocked_by_guardian)
        findViewById<TextView>(R.id.blockReason).setText(reasonRes)

        findViewById<Button>(R.id.goHome).setOnClickListener { goHome() }
    }

    override fun onBackPressed() = goHome()

    private fun goHome() {
        startActivity(Intent(Intent.ACTION_MAIN).addCategory(Intent.CATEGORY_HOME)
            .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK))
        finish()
    }

    companion object {
        const val EXTRA_REASON_RES = "reason_res"
        const val EXTRA_PACKAGE = "package"
    }
}
