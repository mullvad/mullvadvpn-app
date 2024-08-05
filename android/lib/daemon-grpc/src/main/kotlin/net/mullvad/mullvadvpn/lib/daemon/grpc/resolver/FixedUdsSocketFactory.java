package net.mullvad.mullvadvpn.lib.daemon.grpc.resolver;

import android.net.LocalSocketAddress;

import java.io.IOException;
import java.net.InetAddress;
import java.net.InetSocketAddress;
import java.net.Socket;
import java.net.SocketAddress;

import javax.net.SocketFactory;


class FixedUdsSocketFactory extends SocketFactory {

    private final LocalSocketAddress localSocketAddress;

    public FixedUdsSocketFactory(String path, LocalSocketAddress.Namespace namespace) {
        localSocketAddress = new LocalSocketAddress(path, namespace);
    }

    @Override
    public Socket createSocket() throws IOException {
        return create();
    }

    @Override
    public Socket createSocket(String host, int port) throws IOException {
        return createAndConnect();
    }

    @Override
    public Socket createSocket(String host, int port, InetAddress localHost, int localPort)
            throws IOException {
        return createAndConnect();
    }

    @Override
    public Socket createSocket(InetAddress host, int port) throws IOException {
        return createAndConnect();
    }

    @Override
    public Socket createSocket(InetAddress address, int port, InetAddress localAddress, int localPort)
            throws IOException {
        return createAndConnect();
    }

    private Socket create() {
        return new FixedUdsSocket(localSocketAddress);
    }

    private Socket createAndConnect() throws IOException {
        Socket socket = create();
        SocketAddress unusedAddress = new InetSocketAddress(0);
        socket.connect(unusedAddress);
        return socket;
    }
}

