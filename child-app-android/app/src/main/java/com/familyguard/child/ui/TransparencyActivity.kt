package com.familyguard.child.ui

import android.app.Activity
import android.os.Bundle
import android.widget.TextView
import com.familyguard.child.R

/**
 * Child-facing "what my guardian can see" screen. Reinforces transparency: the
 * child can always open this from the persistent notification to understand
 * exactly what is shared. This is a trust feature, not an afterthought.
 */
class TransparencyActivity : Activity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_transparency)
        findViewById<TextView>(R.id.transparencyBody).text = getString(R.string.disclosure_body)
    }
}
