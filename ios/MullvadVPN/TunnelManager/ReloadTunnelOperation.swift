//
//  ReloadTunnelOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 10/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

class ReloadTunnelOperation: ResultOperation<(), TunnelManager.Error> {
    private let state: TunnelManager.State
    private var task: Cancellable?

    init(
        queue: DispatchQueue,
        state: TunnelManager.State,
        completionHandler: @escaping CompletionHandler
    )
    {
        self.state = state

        super.init(
            dispatchQueue: queue,
            completionQueue: queue,
            completionHandler: completionHandler
        )
    }

    override func main() {
        guard let tunnel = self.state.tunnel else {
            finish(completion: .failure(.unsetAccount))
            return
        }

        let session = TunnelIPC.Session(tunnel: tunnel)

        task = session.reloadTunnelSettings { [weak self] completion in
            guard let self = self else { return }

            self.dispatchQueue.async {
                self.finish(completion: completion.mapError { .reloadTunnel($0) })
            }
        }
    }

    override func operationDidCancel() {
        task?.cancel()
        task = nil
    }
}
