//
//  Socks5Connection.swift
//  MullvadTransport
//
//  Created by pronebird on 19/10/2023.
//

import Foundation
import Network

/// A bidirectional data connection between a local endpoint and remote endpoint over socks proxy.
final class Socks5Connection {
    /// The remote endpoint to which the client wants to establish connection over the socks proxy.
    let remoteServerEndpoint: Socks5Endpoint

    /**
     Initializes a new connection passing data between local and remote TCP connection over the socks proxy.

     - Parameters:
         - queue: the queue on which connection events are delivered.
         - localConnection: the local TCP connection.
         - socksProxyEndpoint: the socks proxy endpoint.
         - remoteServerEndpoint: the remote endpoint to which the client wants to establish connection over the socks proxy.
     */
    init(
        queue: DispatchQueue,
        localConnection: NWConnection,
        socksProxyEndpoint: NWEndpoint,
        remoteServerEndpoint: Socks5Endpoint
    ) {
        self.queue = queue
        self.remoteServerEndpoint = remoteServerEndpoint
        self.localConnection = localConnection
        self.remoteConnection = NWConnection(to: socksProxyEndpoint, using: .tcp)
    }

    /**
     Start establishing a connection.

     The start operation is asynchronous. Calls to start after the first one are ignored.
     */
    func start() {
        queue.async { [self] in
            guard case .initialized = state else { return }

            state = .started

            localConnection.stateUpdateHandler = onLocalConnectionState
            remoteConnection.stateUpdateHandler = onRemoteConnectionState
            localConnection.start(queue: queue)
            remoteConnection.start(queue: queue)
        }
    }

    /**
     Cancel the connection.

     Cancellation is asynchronous. All block handlers are released to break retain cycles once connection moved to stopped state. The object is not meant to be
     reused or restarted after cancellation.

     Calls to cancel after the first one are ignored.
     */
    func cancel() {
        queue.async { [self] in
            cancel(error: nil)
        }
    }

    /**
     Set a handler that receives connection state events.

     It's advised to set the state handler before starting the connection to avoid missing updates to the connection state.

     - Parameter newStateHandler: state handler block.
     */
    func setStateHandler(_ newStateHandler: ((Socks5Connection, State) -> Void)?) {
        queue.async { [self] in
            stateHandler = newStateHandler
        }
    }

    // MARK: - Private

    /// Connection state.
    enum State {
        /// Connection object is initialized. Default state.
        case initialized

        /// Connection is started.
        case started

        /// Connection to socks proxy is initiated.
        case connectionInitiated

        /// Connection object is in stopped state.
        case stopped(Error?)

        /// Returns `true` if connection is in `.stopped` state.
        var isStopped: Bool {
            if case .stopped = self {
                return true
            } else {
                return false
            }
        }
    }

    private let queue: DispatchQueue
    private let localConnection: NWConnection
    private let remoteConnection: NWConnection
    private var stateHandler: ((Socks5Connection, State) -> Void)?
    private var state: State = .initialized {
        didSet {
            stateHandler?(self, state)
        }
    }

    private func cancel(error: Error?) {
        guard !state.isStopped else { return }

        state = .stopped(error)
        stateHandler = nil

        localConnection.cancel()
        remoteConnection.cancel()
    }

    private func onLocalConnectionState(_ connectionState: NWConnection.State) {
        switch connectionState {
        case .setup, .preparing, .cancelled:
            break

        case .ready:
            initiateConnection()

        case let .waiting(error), let .failed(error):
            handleError(Socks5Error.localConnectionFailure(error))

        @unknown default:
            break
        }
    }

    private func onRemoteConnectionState(_ connectionState: NWConnection.State) {
        switch connectionState {
        case .setup, .preparing, .cancelled:
            break

        case .ready:
            initiateConnection()

        case let .waiting(error), let .failed(error):
            handleError(Socks5Error.remoteConnectionFailure(error))

        @unknown default:
            break
        }
    }

    /// Initiate connection to socks proxy if local and remote connections are both ready.
    /// Repeat calls to this method do nothing once connection to socks proxy is initiated.
    private func initiateConnection() {
        guard case .started = state else { return }
        guard case (.ready, .ready) = (localConnection.state, remoteConnection.state) else { return }

        state = .connectionInitiated
        sendHandshake()
    }

    private func handleError(_ error: Error) {
        cancel(error: error)
    }

    /// Start handshake with the socks proxy.
    private func sendHandshake() {
        let handshake = Socks5Handshake()
        let negotiation = Socks5HandshakeNegotiation(
            connection: remoteConnection,
            handshake: handshake,
            onComplete: onHandshake,
            onFailure: handleError
        )
        negotiation.perform()
    }

    /// Handles handshake reply.
    /// Initiates authentication flow if indicated in reply, otherwise starts connection negotiation immediately.
    private func onHandshake(_ reply: Socks5HandshakeReply) {
        switch reply.method {
        case .notRequired:
            connect()

        case .usernamePassword:
            // TODO: handle authentication
            break
        }
    }

    /// Start connection negotiation.
    /// Upon successful negotiation, the client can begin exchanging data with remote server.
    private func connect() {
        let negotiation = Socks5ConnectNegotiation(
            connection: remoteConnection,
            endpoint: remoteServerEndpoint,
            onComplete: { [self] reply in
                if case .succeeded = reply.status {
                    stream()
                } else {
                    handleError(Socks5Error.connectionRejected(reply.status))
                }
            },
            onFailure: handleError
        )
        negotiation.perform()
    }

    /// Start streaming data between local and remote endpoint.
    private func stream() {
        let streamHandler = Socks5DataStreamHandler(
            localConnection: localConnection,
            remoteConnection: remoteConnection
        ) { [self] error in
            self.handleError(error)
        }
        streamHandler.start()
    }
}
