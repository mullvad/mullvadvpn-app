/*
 * Copyright (C) 2018 Square, Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
package net.mullvad.mullvadvpn.lib.daemon.grpc.util

import java.net.InetAddress
import java.net.Socket
import java.net.SocketAddress
import javax.net.SocketFactory
import org.newsclub.net.unix.AFSocketAddress

/** Impersonate TCP-style SocketFactory over UNIX domain sockets. */
class UnixDomainSocketFactory(private val addr: SocketAddress) : SocketFactory() {

    override fun createSocket(): Socket {
        val socket = AFSocketAddress.mapOrFail(addr).getAddressFamily().newSocket()
        socket.forceConnectAddress(addr)
        return socket
    }

    override fun createSocket(host: String?, port: Int): Socket {
        return createSocket()
    }

    override fun createSocket(
        host: String,
        port: Int,
        localHost: InetAddress,
        localPort: Int,
    ): Socket {
        return createSocket(host, port)
    }

    override fun createSocket(host: InetAddress?, port: Int): Socket {
        return createSocket()
    }

    override fun createSocket(
        host: InetAddress?,
        port: Int,
        localAddress: InetAddress?,
        localPort: Int,
    ): Socket {
        return createSocket(host, port)
    }
}
