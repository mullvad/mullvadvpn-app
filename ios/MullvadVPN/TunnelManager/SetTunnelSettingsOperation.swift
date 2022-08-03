//
//  SetTunnelSettingsOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 16/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

class SetTunnelSettingsOperation: ResultOperation<Void, TunnelManager.Error> {
    typealias ModificationHandler = (inout TunnelSettings) -> Void

    private let state: TunnelManager.State
    private let modificationBlock: ModificationHandler

    init(
        dispatchQueue: DispatchQueue,
        state: TunnelManager.State,
        modificationBlock: @escaping ModificationHandler,
        completionHandler: @escaping CompletionHandler
    ) {
        self.state = state
        self.modificationBlock = modificationBlock

        super.init(
            dispatchQueue: dispatchQueue,
            completionQueue: dispatchQueue,
            completionHandler: completionHandler
        )
    }

    override func main() {
        guard let accountToken = state.tunnelInfo?.token else {
            finish(completion: .failure(.unsetAccount))
            return
        }

        let result = TunnelSettingsManager
            .update(searchTerm: .accountToken(accountToken)) { tunnelSettings in
                modificationBlock(&tunnelSettings)
            }

        switch result {
        case let .success(newTunnelSettings):
            state.tunnelInfo?.tunnelSettings = newTunnelSettings
            finish(completion: .success(()))

        case let .failure(error):
            finish(completion: .failure(.updateTunnelSettings(error)))
        }
    }
}
