//
//  ReloadTunnelOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 10/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

class ReloadTunnelOperation: ResultOperation<(), TunnelManager.Error> {
    private let queue: DispatchQueue
    private let state: TunnelManager.State
    private var cancellableTask: Cancellable?

    init(queue: DispatchQueue, state: TunnelManager.State, completionHandler: @escaping CompletionHandler) {
        self.queue = queue
        self.state = state

        super.init(completionQueue: queue, completionHandler: completionHandler)
    }

    override func main() {
        queue.async {
            guard !self.isCancelled else {
                self.finish(completion: .cancelled)
                return
            }

            guard let tunnel = self.state.tunnel else {
                self.finish(completion: .failure(.unsetAccount))
                return
            }

            let session = TunnelIPC.Session(tunnel: tunnel)

            self.cancellableTask = session.reloadTunnelSettings { [weak self] completion in
                guard let self = self else { return }

                self.queue.async {
                    self.finish(completion: completion.mapError { .reloadTunnel($0) })
                }
            }
        }
    }

    override func cancel() {
        super.cancel()

        queue.async {
            self.cancellableTask?.cancel()
            self.cancellableTask = nil
        }
    }
}
