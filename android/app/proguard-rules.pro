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

-keep class io.grpc.** { *; }
-keep class mullvad_daemon.management_interface.** { *; }
-keep class com.google.protobuf.** { *; }

-dontwarn com.google.j2objc.annotations.ReflectionSupport
-dontwarn com.google.j2objc.annotations.RetainedWith
-dontwarn com.squareup.okhttp.CipherSuite
-dontwarn com.squareup.okhttp.ConnectionSpec
-dontwarn com.squareup.okhttp.TlsVersion
-dontwarn javax.naming.NamingEnumeration
-dontwarn javax.naming.NamingException
-dontwarn javax.naming.directory.Attribute
-dontwarn javax.naming.directory.Attributes
-dontwarn javax.naming.directory.DirContext
-dontwarn javax.naming.directory.InitialDirContext

