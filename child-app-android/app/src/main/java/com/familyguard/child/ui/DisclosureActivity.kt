package com.familyguard.child.ui

import android.app.Activity
import android.os.Bundle
import android.widget.Button
import android.widget.TextView
import com.familyguard.child.R

/**
 * Disclosure + consent screen shown on the CHILD device before enrollment
 * completes (Compliance C4). It plainly lists what the guardian can see and
 * control, in age-appropriate language. The child must tap "I understand" to
 * proceed. This screen is part of what makes this a legitimate, transparent
 * parental-control tool rather than covert surveillance.
 *
 * Do NOT add a way to skip or hide this screen.
 */
class DisclosureActivity : Activity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_disclosure)

        findViewById<TextView>(R.id.disclosureBody).text = getString(R.string.disclosure_body)

        findViewById<Button>(R.id.agreeButton).setOnClickListener {
            // TODO(phase1): record consent timestamp, then continue to enrollment binding.
            setResult(RESULT_OK)
            finish()
        }
    }
}
