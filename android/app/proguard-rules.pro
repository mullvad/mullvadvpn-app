# Mullvad daemon FFI/JNI
# See: <repository-root>/mullvad-jni/classes.rs
# Keep all talpid classes as they are used for JNI calls
-keep class net.mullvad.talpid.** { *; }
# These are specific classes used in JNI calls with the daemon
-keep class net.mullvad.mullvadvpn.lib.endpoint.ApiEndpointOverride { *; }
-keep class net.mullvad.mullvadvpn.service.MullvadDaemon { *; }
-keep class net.mullvad.mullvadvpn.service.MullvadVpnService { *; }
# All classes that are used in JNI calls are subclasses of Parcelable
-keep class android.os.Parcelable { *; }
# Common java types used in JNI calls
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

# datastore
-keep class net.mullvad.mullvadvpn.repository.UserPreferences { *; }

# for logging
-keepnames class net.mullvad.mullvadvpn.lib.payment.model.PurchaseResult
-keepnames class net.mullvad.mullvadvpn.lib.payment.model.PaymentAvailability


