//
//  TunnelIPCSession.swift
//  MullvadVPN
//
//  Created by pronebird on 16/09/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import NetworkExtension

extension TunnelIPC {
    /// Wrapper class around `NETunnelProviderSession` that provides convenient interface for
    /// interacting with the Packet Tunnel process.
    final class Session {
        private let connection: VPNConnectionProtocol
        private let queue = DispatchQueue(label: "TunnelIPC.SessionQueue")
        private let operationQueue = OperationQueue()

        init(connection: VPNConnectionProtocol) {
            self.connection = connection
        }

        func reloadTunnelSettings(completionHandler: @escaping (OperationCompletion<(), TunnelIPC.Error>) -> Void) -> Cancellable {
            let operation = RequestOperation(
                queue: queue,
                connection: connection,
                request: .reloadTunnelSettings,
                options: TunnelIPC.RequestOptions(),
                completionHandler: completionHandler
            )

            operationQueue.addOperation(operation)

            return AnyCancellable {
                operation.cancel()
            }
        }

        func getTunnelConnectionInfo(completionHandler: @escaping (OperationCompletion<TunnelConnectionInfo?, TunnelIPC.Error>) -> Void) -> Cancellable {
            let operation = RequestOperation<TunnelConnectionInfo?>(
                queue: queue,
                connection: connection,
                request: .tunnelConnectionInfo,
                options: TunnelIPC.RequestOptions(),
                completionHandler: completionHandler
            )

            operationQueue.addOperation(operation)

            return AnyCancellable {
                operation.cancel()
            }
        }
    }
}
