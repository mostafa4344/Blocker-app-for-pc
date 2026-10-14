# Child Android App (single app on the child's phone)

Kotlin. This is the one app installed on the child's device. It:
- shows a **mandatory persistent notification** whenever it is active (Play compliance),
- shows a **disclosure/consent screen** during setup,
- enrolls against the backend (QR → device JWT),
- enforces the policy locally (`PolicyEngine`),
- resists casual removal via **Device Admin** (and, when provisioned, **Device Owner**).

## ⚠️ Non-negotiable compliance rules (see ../docs/03-PLAY-STORE-COMPLIANCE.md)
- The persistent notification must **never** be removable while the app is active.
- The app icon must **always** be visible. There is no "hide icon" feature, ever.
- No "spy/secret/stealth" language anywhere. This is a parental/family tool.
- No capture of message/call content, keystrokes, ambient audio/camera.
- Emergency calls are never blocked.

## Status
**Skeleton.** Structure and the key compliance-critical pieces (persistent notification,
disclosure, device-admin hook) are stubbed with real intent. Enrollment, policy enforcement,
sync, and Device Owner provisioning are **designed but not yet implemented** — see TODOs and
`../docs/05-ROADMAP.md`.

## Module layout (planned)
```
app/src/main/
├── AndroidManifest.xml
├── java/com/familyguard/child/
│   ├── FamilyGuardApp.kt            # Application; notification channels
│   ├── admin/GuardDeviceAdminReceiver.kt   # Device Admin / removal resistance
│   ├── service/GuardForegroundService.kt   # persistent notification (COMPLIANCE)
│   ├── ui/DisclosureActivity.kt     # consent screen shown before enrollment
│   ├── ui/EnrollmentActivity.kt     # scan QR, call enroll-device, store JWT
│   ├── ui/TransparencyActivity.kt   # child-facing "what my guardian can see"
│   ├── policy/PolicyEngine.kt       # local enforcement (app blocking, limits)
│   ├── policy/PolicyStore.kt        # cached policy (offline-first)
│   ├── net/Api.kt                   # device-sync / heartbeat calls (device JWT)
│   ├── net/SecureStore.kt           # EncryptedSharedPreferences for the device JWT
│   └── work/SyncWorker.kt           # WorkManager periodic sync + FCM wake
└── res/
    ├── xml/device_admin.xml         # device admin policies
    └── values/strings.xml           # disclosure + notification copy
```

## Build
Standard Android Studio project (Gradle KTS). Set `BACKEND_FUNCTIONS_URL` in `local.properties`
or BuildConfig to your Supabase functions base URL. Requires a `google-services.json` for FCM
(not committed).

## Device Owner vs Device Admin (removal resistance)
- **Device Admin** (implemented hook): the app registers as a device administrator; the user is
  nudged and it's harder to remove, but a determined user still can from Settings.
- **Device Owner** (planned): strongest; must be provisioned at **new-device/factory-reset** setup
  via QR/NFC/zero-touch. Android then blocks uninstall/disable. Cannot be applied to an
  already-set-up phone. Still removable via recovery/reflash — document honestly, don't promise
  "undeletable".
