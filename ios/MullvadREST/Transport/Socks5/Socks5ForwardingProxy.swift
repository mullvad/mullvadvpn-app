//
//  Socks5ForwardingProxy.swift
//  MullvadTransport
//
//  Created by pronebird on 18/10/2023.
//

import Foundation
import Network

/**
 The proxy that can forward data connection from local TCP port to remote TCP server over the socks proxy.

 The forwarding socks proxy acts as a transparent proxy. The HTTP/S clients that don't support proxy configuration can be configured to direct their traffic at the
 local TCP port opened by the forwarding socks proxy.

 The forwarding proxy then takes care of negotiating with the remote socks proxy and transparently handles all traffic as if the HTTP/S client talks directly to the remote
 server.

 Refer to RFC1928 for more info on socks5: <https://datatracker.ietf.org/doc/html/rfc1928>
 */
public final class Socks5ForwardingProxy {
    /// Socks proxy endpoint.
    public let socksProxyEndpoint: NWEndpoint

    /// Remote server that socks proxy should connect to.
    public let remoteServerEndpoint: Socks5Endpoint

    public let configuration: Socks5Configuration

    /// Local TCP port that clients should use to communicate with the remote server.
    /// This property is set once the proxy is successfully started.
    public var listenPort: UInt16? {
        queue.sync {
            switch state {
            case let .started(listener, _):
                return listener.port?.rawValue
            case .stopped, .starting:
                return nil
            }
        }
    }

    /**
     Initializes a socks forwarding proxy accepting connections on local TCP port and establishing connection to the remote endpoint over socks proxy.

     - Parameters:
       - socksProxyEndpoint: socks proxy endpoint.
       - remoteServerEndpoint: remote server that socks proxy should connect to.
     */
    public init(
        socksProxyEndpoint: NWEndpoint,
        remoteServerEndpoint: Socks5Endpoint,
        configuration: Socks5Configuration
    ) {
        self.socksProxyEndpoint = socksProxyEndpoint
        self.remoteServerEndpoint = remoteServerEndpoint
        self.configuration = configuration
    }

    deinit {
        stopInner()
    }

    /**
     Start forwarding proxy.

     Repeat calls do nothing, but accumulate the completion handler for invocation once the proxy moves to the next state.

     - Parameter completion: completion handler that is called once the TCP listener is ready in the first time or failed before moving to the ready state.
                             Invoked on main queue.
     */
    public func start(completion: @escaping (Error?) -> Void) {
        queue.async {
            self.startListener { error in
                DispatchQueue.main.async {
                    completion(error)
                }
            }
        }
    }

    /**
     Stop forwarding proxy.

     - Parameter completion: completion handler that's called immediately after cancelling the TCP listener. Invoked on main queue.
     */
    public func stop(completion: (() -> Void)? = nil) {
        queue.async {
            self.stopInner()

            DispatchQueue.main.async {
                completion?()
            }
        }
    }

    /**
     Set error handler to receive unrecoverable errors at runtime.

     - Parameter errorHandler: an error handler block. Invoked on main queue.
     */
    public func setErrorHandler(_ errorHandler: ((Error) -> Void)?) {
        queue.async {
            self.errorHandler = errorHandler
        }
    }

    // MARK: - Private

    private enum State {
        /// Proxy is starting up.
        case starting(listener: NWListener, completion: (Error?) -> Void)

        /// Proxy is ready.
        case started(listener: NWListener, openConnections: [Socks5Connection])

        /// Proxy is not running.
        case stopped
    }

    private let queue = DispatchQueue(label: "Socks5ForwardingProxy-queue")
    private var state: State = .stopped
    private var errorHandler: ((Error) -> Void)?

    /**
     Start TCP listener.

     - Parameter completion: completion handler that is called once the TCP listener is ready or failed.
     */
    private func startListener(completion: @escaping (Error?) -> Void) {
        switch state {
        case .started:
            completion(nil)

        case let .starting(listener, previousCompletion):
            // Accumulate completion handlers when requested to start multiple times in a row.
            self.state = .starting(listener: listener, completion: { error in
                previousCompletion(error)
                completion(error)
            })

        case .stopped:
            do {
                let tcpListener = try makeTCPListener()
                state = .starting(listener: tcpListener, completion: completion)
                tcpListener.start(queue: queue)
            } catch {
                completion(Socks5Error.createTcpListener(error))
            }
        }
    }

    /**
     Create new TCP listener.

     - Throws: an instance of `NWError` if unable to initialize `NWListener`.
     - Returns: a configured instance of `NWListener`.
     */
    private func makeTCPListener() throws -> NWListener {
        let tcpListener = try NWListener(using: .tcp)
        tcpListener.stateUpdateHandler = { [weak self] state in
            self?.onListenerState(state)
        }
        tcpListener.newConnectionHandler = { [weak self] connection in
            self?.onNewConnection(connection)
        }
        return tcpListener
    }

    /**
     Reset block handlers and cancel an instance of `NWListener`.

     - Parameter tcpListener: an instance of `NWListener`.
     */
    private func cancelListener(_ tcpListener: NWListener) {
        tcpListener.stateUpdateHandler = nil
        tcpListener.newConnectionHandler = nil
        tcpListener.cancel()
    }

    private func stopInner() {
        switch state {
        case let .starting(listener, completion):
            state = .stopped
            cancelListener(listener)
            DispatchQueue.main.async {
                completion(Socks5Error.cancelledDuringStartup)
            }

        case let .started(listener, openConnections):
            state = .stopped
            cancelListener(listener)
            openConnections.forEach { $0.cancel() }

        case .stopped:
            break
        }
    }

    private func onReady() {
        switch state {
        case let .starting(listener, completion):
            state = .started(listener: listener, openConnections: [])

            DispatchQueue.main.async {
                completion(nil)
            }

        case .started, .stopped:
            break
        }
    }

    private func onFailure(_ error: Error) {
        switch state {
        case let .starting(_, completion):
            state = .stopped

            DispatchQueue.main.async {
                completion(error)
            }

        case .started:
            state = .stopped
            DispatchQueue.main.async {
                self.errorHandler?(error)
            }

        case .stopped:
            break
        }
    }

    private func onListenerState(_ listenerState: NWListener.State) {
        switch listenerState {
        case .setup, .cancelled:
            break

        case .ready:
            onReady()

        case let .failed(error), let .waiting(error):
            onFailure(error)

        @unknown default:
            break
        }
    }

    private func onNewConnection(_ connection: NWConnection) {
        switch state {
        case .starting, .stopped:
            connection.cancel()

        case .started(let listener, var openConnections):
            let socks5Connection = Socks5Connection(
                queue: queue,
                localConnection: connection,
                socksProxyEndpoint: socksProxyEndpoint,
                remoteServerEndpoint: remoteServerEndpoint,
                configuration: configuration
            )
            socks5Connection.setStateHandler { [weak self] socks5Connection, state in
                if case let .stopped(error) = state {
                    self?.onEndConnection(socks5Connection, error: error)
                }
            }

            openConnections.append(socks5Connection)
            state = .started(listener: listener, openConnections: openConnections)

            socks5Connection.start()
        }
    }

    private func onEndConnection(_ connection: Socks5Connection, error: Error?) {
        switch state {
        case .stopped, .starting:
            break

        case .started(let listener, var openConnections):
            guard let index = openConnections.firstIndex(where: { $0 === connection }) else { return }

            openConnections.remove(at: index)
            state = .started(listener: listener, openConnections: openConnections)
        }
    }
}
