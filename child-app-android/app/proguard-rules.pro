# FamilyGuard ProGuard rules
# Keep OkHttp/okio platform-optional classes quiet.
-dontwarn okhttp3.**
-dontwarn okio.**
-dontwarn org.conscrypt.**
# ZXing embedded
-keep class com.journeyapps.** { *; }
-keep class com.google.zxing.** { *; }
