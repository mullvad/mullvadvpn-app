//
//  SetTunnelSettingsOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 16/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

class SetTunnelSettingsOperation: AsyncOperation {
    typealias ModificationHandler = (inout TunnelSettings) -> Void
    typealias CompletionHandler = (OperationCompletion<(), TunnelManager.Error>) -> Void

    private let queue: DispatchQueue
    private let state: TunnelManager.State
    private let modificationBlock: ModificationHandler
    private var completionHandler: CompletionHandler?

    init(queue: DispatchQueue, state: TunnelManager.State, modificationBlock: @escaping ModificationHandler, completionHandler: @escaping CompletionHandler) {
        self.queue = queue
        self.state = state
        self.modificationBlock = modificationBlock
        self.completionHandler = completionHandler
    }

    override func main() {
        queue.async {
            self.execute { completion in
                self.completionHandler?(completion)
                self.completionHandler = nil

                self.finish()
            }
        }
    }

    private func execute(completionHandler: CompletionHandler) {
        guard !isCancelled else {
            completionHandler(.cancelled)
            return
        }

        guard let accountToken = state.tunnelInfo?.token else {
            completionHandler(.failure(.unsetAccount))
            return
        }

        let result = TunnelSettingsManager.update(searchTerm: .accountToken(accountToken)) { tunnelSettings in
            self.modificationBlock(&tunnelSettings)
        }

        switch result {
        case .success(let newTunnelSettings):
            state.tunnelInfo?.tunnelSettings = newTunnelSettings

            completionHandler(.success(()))

        case .failure(let error):
            completionHandler(.failure(.updateTunnelSettings(error)))

        }
    }
}
