# Mullvad
# Keeping all Mullvad classes etc until the project has been split into multiple sub-projects
# where it's better defined where the FFI/JNI boundaries are.
-keep class net.mullvad.** { *; }

# Mullvad daemon FFI/JNI
# See: <repository-root>/mullvad-jni/classes.rs
-keep class android.os.Parcelable { *; }
-keep class java.lang.Boolean { *; }
-keep class java.lang.Integer { *; }
-keep class java.lang.String { *; }
-keep class java.net.InetAddress { *; }
-keep class java.net.InetSocketAddress { *; }
-keep class java.util.ArrayList { *; }

# Joda Time
-dontwarn org.joda.convert.**
-dontwarn org.joda.time.**
-keep class org.joda.time.** { *; }
-keep interface org.joda.time.** { *; }
