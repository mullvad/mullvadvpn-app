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

# grpc
-keep class io.grpc.okhttp.OkHttpChannelBuilder { *; }
-keep class mullvad_daemon.management_interface.** { *; }
-keep class com.google.protobuf.Timestamp { *; }
-keepnames class com.google.protobuf.** { *; }
-dontwarn com.google.j2objc.annotations.ReflectionSupport
-dontwarn com.google.j2objc.annotations.RetainedWith
-dontwarn com.squareup.okhttp.CipherSuite
-dontwarn com.squareup.okhttp.ConnectionSpec
-dontwarn com.squareup.okhttp.TlsVersion


