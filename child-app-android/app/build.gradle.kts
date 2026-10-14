plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    // Enable once google-services.json is added:
    // id("com.google.gms.google-services")
}

android {
    namespace = "com.familyguard.child"
    compileSdk = 35

    defaultConfig {
        applicationId = "com.familyguard.child"
        minSdk = 26              // Android 8.0 — needed for notification channels / modern FGS
        targetSdk = 35           // keep current per Play target-API requirement
        versionCode = 1
        versionName = "0.1.0"

        // Base URL of your deployed Supabase Edge Functions (no trailing slash).
        // Override in local.properties -> gradle reads it into BuildConfig.
        buildConfigField("String", "FUNCTIONS_URL",
            "\"${project.findProperty("FUNCTIONS_URL") ?: "https://YOUR-PROJECT.supabase.co/functions/v1"}\"")
    }

    buildFeatures { buildConfig = true }

    buildTypes {
        release {
            isMinifyEnabled = true
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    kotlinOptions { jvmTarget = "17" }
}

dependencies {
    implementation("androidx.core:core-ktx:1.13.1")
    implementation("androidx.appcompat:appcompat:1.7.0")
    implementation("com.google.android.material:material:1.12.0")
    implementation("androidx.work:work-runtime-ktx:2.9.1")
    implementation("androidx.security:security-crypto:1.1.0-alpha06") // EncryptedSharedPreferences
    implementation("com.squareup.okhttp3:okhttp:4.12.0")
    implementation("org.json:json:20240303")
    // QR scanning for enrollment:
    implementation("com.journeyapps:zxing-android-embedded:4.3.0")
    // FCM (add google-services.json + enable the plugin above):
    // implementation(platform("com.google.firebase:firebase-bom:33.3.0"))
    // implementation("com.google.firebase:firebase-messaging-ktx")
}
