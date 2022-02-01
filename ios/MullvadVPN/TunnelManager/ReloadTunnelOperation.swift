//
//  ReloadTunnelOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 10/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

class ReloadTunnelOperation: AsyncOperation {
    typealias CompletionHandler = (OperationCompletion<(), TunnelManager.Error>) -> Void

    private let queue: DispatchQueue
    private let state: TunnelManager.State
    private var request: Cancellable?
    private var completionHandler: CompletionHandler?

    init(queue: DispatchQueue, state: TunnelManager.State, completionHandler: @escaping CompletionHandler) {
        self.queue = queue
        self.state = state
        self.completionHandler = completionHandler
    }

    override func main() {
        queue.async {
            guard !self.isCancelled else {
                self.completeOperation(completion: .cancelled)
                return
            }

            guard let tunnelProvider = self.state.tunnelProvider else {
                self.completeOperation(completion: .failure(.missingAccount))
                return
            }

            let session = TunnelIPC.Session(connection: tunnelProvider.connection)

            self.request = session.reloadTunnelSettings { [weak self] completion in
                guard let self = self else { return }

                self.queue.async {
                    self.completeOperation(completion: completion.mapError { .reloadTunnel($0) })
                }
            }
        }
    }

    override func cancel() {
        super.cancel()

        queue.async {
            self.request?.cancel()
        }
    }

    private func completeOperation(completion: OperationCompletion<(), TunnelManager.Error>) {
        completionHandler?(completion)
        completionHandler = nil

        finish()
    }

}
