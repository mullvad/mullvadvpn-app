package net.mullvad.mullvadvpn.lib.daemon.grpc.resolver

import android.net.LocalSocketAddress
import io.grpc.ChannelCredentials
import io.grpc.ExperimentalApi
import io.grpc.InsecureChannelCredentials
import io.grpc.ManagedChannelBuilder
import java.lang.reflect.InvocationTargetException
import javax.net.SocketFactory

@ExperimentalApi("A stopgap. Not intended to be stabilized")
object FixedUdsChannelBuilder {
    private val OKHTTP_CHANNEL_BUILDER_CLASS = findOkHttp()

    private fun findOkHttp(): Class<out ManagedChannelBuilder<*>>? {
        return try {
            Class.forName("io.grpc.okhttp.OkHttpChannelBuilder")
                .asSubclass(ManagedChannelBuilder::class.java)
        } catch (e: ClassNotFoundException) {
            null
        }
    }

    /**
     * Returns a channel to the UDS endpoint specified by the file-path.
     *
     * @param path unix file system path to use for Unix Domain Socket.
     * @param namespace the type of the namespace that the path belongs to.
     */
    fun forPath(path: String?, namespace: LocalSocketAddress.Namespace?): ManagedChannelBuilder<*> {
        if (OKHTTP_CHANNEL_BUILDER_CLASS == null) {
            throw UnsupportedOperationException("OkHttpChannelBuilder not found on the classpath")
        }
        try {
            // Target 'dns:///localhost' is unused, but necessary as an argument for
            // OkHttpChannelBuilder.
            // TLS is unsupported because Conscrypt assumes the platform Socket implementation to
            // improve
            // performance by using the file descriptor directly.
            val o =
                OKHTTP_CHANNEL_BUILDER_CLASS.getMethod(
                        "forTarget",
                        String::class.java,
                        ChannelCredentials::class.java
                    )
                    .invoke(null, "unix:///$path", InsecureChannelCredentials.create())
            val builder = OKHTTP_CHANNEL_BUILDER_CLASS.cast(o)
            OKHTTP_CHANNEL_BUILDER_CLASS.getMethod("socketFactory", SocketFactory::class.java)
                .invoke(builder, FixedUdsSocketFactory(path, namespace))
            return builder
        } catch (e: IllegalAccessException) {
            throw RuntimeException("Failed to create OkHttpChannelBuilder", e)
        } catch (e: NoSuchMethodException) {
            throw RuntimeException("Failed to create OkHttpChannelBuilder", e)
        } catch (e: InvocationTargetException) {
            throw RuntimeException("Failed to create OkHttpChannelBuilder", e)
        }
    }
}
