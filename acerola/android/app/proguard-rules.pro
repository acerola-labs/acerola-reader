# ==============================================================================
# DEFINITIVE PROGUARD/R8 CONFIGURATION - ACEROLA
# Focus: full stability (Parcelable, reflection on our own DTOs, entry points)
# ==============================================================================

# 1. Essential Kotlin metadata
-keepattributes *Annotation*, Signature, Exceptions, InnerClasses, EnclosingMethod, SourceFile, LineNumberTable
-keep class kotlin.Metadata { *; }

# 2. Android entry points (keep everything)
-keep public class * extends android.app.Activity
-keep public class * extends android.app.Service
-keep public class * extends android.app.Application
-keep public class * extends androidx.lifecycle.ViewModel
-keep public class * extends android.content.ContentProvider
-keep public class * extends android.content.BroadcastReceiver
-keepclasseswithmembers class * {
    public <init>(android.content.Context, android.util.AttributeSet);
    public <init>(android.content.Context, android.util.AttributeSet, int);
}

# 3. Parcelable and DTOs (app-specific preservation)
# Keeps the app's Parcelable classes intact.
-keep class * implements android.os.Parcelable {
    public static final ** CREATOR;
    <fields>;
    <methods>;
}
# Allows internal optimizations but preserves the DTOs' serialization structure.
-keepclassmembers,allowoptimization class br.acerola.comic.dto.** { *; }
-keepclassmembers,allowoptimization class br.acerola.comic.data.remote.dto.** { *; }

# 4. Hilt (dependency injection)
-keepclassmembers class * {
    @dagger.hilt.android.AndroidEntryPoint *;
    @javax.inject.Inject *;
}

-keep class com.sun.jna.** { *; }
-keepclassmembers class com.sun.jna.** { *; }
-dontwarn com.sun.jna.**

-keep class p2p.** { *; }
-keepclassmembers class p2p.** { *; }