//
//  SetTunnelSettingsOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 16/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

class SetTunnelSettingsOperation: ResultOperation<(), TunnelManager.Error> {
    typealias ModificationHandler = (inout TunnelSettings) -> Void

    private let queue: DispatchQueue
    private let state: TunnelManager.State
    private let modificationBlock: ModificationHandler

    init(queue: DispatchQueue, state: TunnelManager.State, modificationBlock: @escaping ModificationHandler, completionHandler: @escaping CompletionHandler) {
        self.queue = queue
        self.state = state
        self.modificationBlock = modificationBlock

        super.init(completionQueue: queue, completionHandler: completionHandler)
    }

    override func main() {
        queue.async {
            guard !self.isCancelled else {
                self.finish(completion: .cancelled)
                return
            }

            guard let accountToken = self.state.tunnelInfo?.token else {
                self.finish(completion: .failure(.unsetAccount))
                return
            }

            let result = TunnelSettingsManager.update(searchTerm: .accountToken(accountToken)) { tunnelSettings in
                self.modificationBlock(&tunnelSettings)
            }

            switch result {
            case .success(let newTunnelSettings):
                self.state.tunnelInfo?.tunnelSettings = newTunnelSettings
                self.finish(completion: .success(()))

            case .failure(let error):
                self.finish(completion: .failure(.updateTunnelSettings(error)))
            }
        }
    }
}
