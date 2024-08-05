package net.mullvad.mullvadvpn.lib.daemon.grpc.resolver;

import android.net.LocalSocketAddress;

import java.lang.reflect.InvocationTargetException;

import javax.annotation.Nullable;
import javax.net.SocketFactory;

import io.grpc.ChannelCredentials;
import io.grpc.ExperimentalApi;
import io.grpc.InsecureChannelCredentials;
import io.grpc.ManagedChannelBuilder;

@ExperimentalApi("A stopgap. Not intended to be stabilized")
public final class FixedUdsChannelBuilder {
    @Nullable
    @SuppressWarnings("rawtypes")
    private static final Class<? extends ManagedChannelBuilder> OKHTTP_CHANNEL_BUILDER_CLASS =
            findOkHttp();

    @SuppressWarnings("rawtypes")
    private static Class<? extends ManagedChannelBuilder> findOkHttp() {
        try {
            return Class.forName("io.grpc.okhttp.OkHttpChannelBuilder")
                    .asSubclass(ManagedChannelBuilder.class);
        } catch (ClassNotFoundException e) {
            return null;
        }
    }

    /**
     * Returns a channel to the UDS endpoint specified by the file-path.
     *
     * @param path unix file system path to use for Unix Domain Socket.
     * @param namespace the type of the namespace that the path belongs to.
     */
    public static ManagedChannelBuilder<?> forPath(String path, LocalSocketAddress.Namespace namespace) {
        if (OKHTTP_CHANNEL_BUILDER_CLASS == null) {
            throw new UnsupportedOperationException("OkHttpChannelBuilder not found on the classpath");
        }
        try {
            // Target 'dns:///localhost' is unused, but necessary as an argument for OkHttpChannelBuilder.
            // TLS is unsupported because Conscrypt assumes the platform Socket implementation to improve
            // performance by using the file descriptor directly.
            Object o = OKHTTP_CHANNEL_BUILDER_CLASS
                    .getMethod("forTarget", String.class, ChannelCredentials.class)
                    .invoke(null, "dns:///localhost", InsecureChannelCredentials.create());
            ManagedChannelBuilder<?> builder = OKHTTP_CHANNEL_BUILDER_CLASS.cast(o);
            OKHTTP_CHANNEL_BUILDER_CLASS
                    .getMethod("socketFactory", SocketFactory.class)
                    .invoke(builder, new FixedUdsSocketFactory(path, namespace));
            return builder;
        } catch (IllegalAccessException e) {
            throw new RuntimeException("Failed to create OkHttpChannelBuilder", e);
        } catch (NoSuchMethodException e) {
            throw new RuntimeException("Failed to create OkHttpChannelBuilder", e);
        } catch (InvocationTargetException e) {
            throw new RuntimeException("Failed to create OkHttpChannelBuilder", e);
        }
    }

    private FixedUdsChannelBuilder() {}
}

