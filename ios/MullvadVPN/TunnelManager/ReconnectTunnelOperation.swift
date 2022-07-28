//
//  ReconnectTunnelOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 10/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

class ReconnectTunnelOperation: ResultOperation<(), TunnelManager.Error> {
    private let state: TunnelManager.State
    private let selectNewRelay: Bool
    private var task: Cancellable?

    init(
        dispatchQueue: DispatchQueue,
        state: TunnelManager.State,
        selectNewRelay: Bool
    )
    {
        self.state = state
        self.selectNewRelay = selectNewRelay

        super.init(dispatchQueue: dispatchQueue)
    }

    override func main() {
        guard let tunnel = self.state.tunnel,
              let relayConstraints = state.tunnelSettings?.relayConstraints else {
            finish(completion: .failure(.unsetTunnel))
            return
        }

        do {
            var selectorResult: RelaySelectorResult?

            if selectNewRelay {
                let cachedRelays = try RelayCache.Tracker.shared.getCachedRelays()
                selectorResult = try RelaySelector.evaluate(
                    relays: cachedRelays.relays,
                    constraints: relayConstraints
                )
            }

            task = tunnel.reconnectTunnel(
                relaySelectorResult: selectorResult
            ) { [weak self] completion in
                self?.finish(completion: completion.mapError { .reloadTunnel($0) })
            }
        } catch {
            finish(completion: .failure(.reloadTunnel(error)))
        }
    }

    override func operationDidCancel() {
        task?.cancel()
        task = nil
    }
}
