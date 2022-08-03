//
//  ReconnectTunnelOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 10/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

class ReconnectTunnelOperation: ResultOperation<Void, Error> {
    private let interactor: TunnelInteractor
    private let selectNewRelay: Bool
    private var task: Cancellable?

    init(
        dispatchQueue: DispatchQueue,
        interactor: TunnelInteractor,
        selectNewRelay: Bool
    ) {
        self.interactor = interactor
        self.selectNewRelay = selectNewRelay

        super.init(dispatchQueue: dispatchQueue)
    }

    override func main() {
        guard let tunnel = interactor.tunnel else {
            finish(completion: .failure(UnsetTunnelError()))
            return
        }

        do {
            var selectorResult: RelaySelectorResult?

            if selectNewRelay {
                let cachedRelays = try RelayCache.Tracker.shared.getCachedRelays()
                selectorResult = try RelaySelector.evaluate(
                    relays: cachedRelays.relays,
                    constraints: interactor.settings.relayConstraints
                )
            }

            task = tunnel.reconnectTunnel(
                relaySelectorResult: selectorResult
            ) { [weak self] completion in
                self?.finish(completion: completion)
            }
        } catch {
            finish(completion: .failure(error))
        }
    }

    override func operationDidCancel() {
        task?.cancel()
        task = nil
    }
}
