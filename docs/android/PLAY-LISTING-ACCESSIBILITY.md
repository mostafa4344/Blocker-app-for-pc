# Google Play: long description + Accessibility disclosure

Rejection (2026-10): "app description does not reflect the use of AccessibilityService API".
Fix = paste the long description below in Play Console → Store presence → Main store listing,
update the Accessibility declaration video, then resubmit. Edit the feature list to match
what the Android build really does.

## Long description (paste as-is, < 4000 chars)

Content Guard helps you stay away from harmful and distracting content on your phone. Block adult websites, distracting apps and keywords, set schedules and daily limits, and keep your focus.

KEY FEATURES
• Block adult and harmful websites in your browser
• Block apps you choose, all day or on a schedule
• Block keywords in browser address bars and search
• Force SafeSearch on supported search engines
• Schedules and daily time limits
• Optional protection mode that you turn on yourself, with a waiting time before it can be turned off

ACCESSIBILITY SERVICE USE (IMPORTANT)
Content Guard uses Android's AccessibilityService API. This is a core feature of the app. Blocking does not work without it.

What we use it for:
• Read the website address (URL) and search text shown in supported browsers, so that blocked websites and keywords can be detected and closed.
• Detect which app is open on screen, so that apps you chose to block can be closed and a block screen shown.
• Detect when someone tries to change Content Guard's protection settings while protection mode, which you turned on yourself, is active.

What we do NOT do:
• We do not collect, store, or send your screen content, typed text, passwords, messages, or browsing history to our servers or to anyone else. All checks happen on your device.
• We do not use the Accessibility API to change your settings without your permission.
• We do not record calls or audio.
• We do not stop you from turning off the Accessibility permission in Android Settings.

Your consent:
Before turning on the Accessibility permission, the app shows a clear screen that explains how the service is used. The service is only turned on if you tap "Agree" and enable it yourself in Android Settings. You can turn it off at any time in Settings → Accessibility → Content Guard.

PRIVACY
Privacy policy: <your privacy policy URL>

Content Guard is not an accessibility tool for people with disabilities. It uses the Accessibility API only for the content blocking features described above.

## Also required (or Google will reject again)

1. **Prominent disclosure screen inside the app** (shown before sending the user to
   Accessibility settings). It must say the same thing as above, have "Agree" and
   "No thanks" buttons, and must not be skippable or hidden in the privacy policy.
2. **Video** (Play Console → App content → Accessibility API): YouTube link (unlisted is fine),
   about 30–90 s, showing: the disclosure screen → user taps Agree → enabling the service in
   Android Settings → a blocked website/app being closed by the app.
3. **Do NOT set `android:isAccessibilityTool="true"`** in the service XML.
4. **Accessibility declaration form** in Play Console must match the description.
5. **Uninstall/settings protection is risky.** The policy forbids using Accessibility to stop
   users from uninstalling or disabling apps, except in a parental-control app authorized by a
   parent. If the Android app blocks uninstalling or blocks the Accessibility settings page,
   either remove that, or make it clearly opt-in with a timed exit (like the PC app's cooldown)
   and describe it in the listing and the video. Safest for approval: remove it for this
   submission.
6. Privacy policy must also mention Accessibility use and that no screen data leaves the device.
