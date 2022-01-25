//
//  SetTunnelSettingsOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 16/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

protocol SetTunnelSettingsOperationDelegate: AnyObject {
    func operationDidRequestTunnelInfo(_ operation: Operation) -> TunnelInfo?
    func operation(_ operation: Operation, didFinishSettingTunnelSettingsWithCompletion completion: OperationCompletion<TunnelSettings, TunnelManager.Error>)
}

class SetTunnelSettingsOperation: BaseTunnelOperation<TunnelSettings, TunnelManager.Error> {
    private weak var delegate: SetTunnelSettingsOperationDelegate?
    private let modificationBlock: (inout TunnelSettings) -> Void

    init(queue: DispatchQueue, delegate: SetTunnelSettingsOperationDelegate, modificationBlock: @escaping (inout TunnelSettings) -> Void) {
        self.delegate = delegate
        self.modificationBlock = modificationBlock

        super.init(queue: queue)
    }

    override func main() {
        queue.async {
            guard !self.isCancelled else {
                self.completeOperation(completion: .cancelled)
                return
            }

            guard let token = self.delegate?.operationDidRequestTunnelInfo(self)?.token else {
                self.completeOperation(completion: .failure(.missingAccount))
                return
            }

            let result = TunnelSettingsManager.update(searchTerm: .accountToken(token)) { tunnelSettings in
                self.modificationBlock(&tunnelSettings)
            }

            let mappedResult = result.mapError { TunnelManager.Error.updateTunnelSettings($0) }

            self.completeOperation(completion: OperationCompletion(result: mappedResult))
        }
    }

    override func completeOperation(completion: OperationCompletion<TunnelSettings, TunnelManager.Error>) {
        delegate?.operation(self, didFinishSettingTunnelSettingsWithCompletion: completion)
        delegate = nil

        super.completeOperation(completion: completion)
    }
}
