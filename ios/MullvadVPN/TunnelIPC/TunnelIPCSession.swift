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
        private let tunnel: Tunnel
        private let queue = DispatchQueue(label: "TunnelIPC.SessionQueue")
        private let operationQueue = OperationQueue()

        init(tunnel: Tunnel) {
            self.tunnel = tunnel
        }

        func reloadTunnelSettings(completionHandler: @escaping (OperationCompletion<(), TunnelIPC.Error>) -> Void) -> Cancellable {
            let operation = RequestOperation(
                queue: queue,
                tunnel: tunnel,
                request: .reloadTunnelSettings,
                options: TunnelIPC.RequestOptions(),
                completionHandler: completionHandler
            )

            operationQueue.addOperation(operation)

            return AnyCancellable {
                operation.cancel()
            }
        }

        func getTunnelStatus(completionHandler: @escaping (OperationCompletion<PacketTunnelStatus, TunnelIPC.Error>) -> Void) -> Cancellable {
            let operation = RequestOperation<PacketTunnelStatus>(
                queue: queue,
                tunnel: tunnel,
                request: .getTunnelStatus,
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
