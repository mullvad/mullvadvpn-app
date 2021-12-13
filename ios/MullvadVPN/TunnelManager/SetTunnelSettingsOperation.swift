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

    func operation(_ operation: Operation, didSetTunnelSettings newTunnelSettings: TunnelSettings)
    func operation(_ operation: Operation, didFailToSetTunnelSettingsWithError error: TunnelManager.Error)
}

class SetTunnelSettingsOperation: AsyncOperation {
    typealias CompletionHandler = (TunnelManager.Error?) -> Void

    private let queue: DispatchQueue
    private weak var delegate: SetTunnelSettingsOperationDelegate?
    private let modificationBlock: (inout TunnelSettings) -> Void
    private var completionHandler: CompletionHandler?

    init(queue: DispatchQueue, delegate: SetTunnelSettingsOperationDelegate, modificationBlock: @escaping (inout TunnelSettings) -> Void, completionHandler: CompletionHandler?) {
        self.queue = queue
        self.delegate = delegate
        self.modificationBlock = modificationBlock
        self.completionHandler = completionHandler
    }

    override func main() {
        queue.async {
            guard !self.isCancelled else {
                self.finish(error: nil)
                return
            }

            guard let token = self.delegate?.operationDidRequestTunnelInfo(self)?.token else {
                let tunnelManagerError = TunnelManager.Error.missingAccount
                self.delegate?.operation(self, didFailToSetTunnelSettingsWithError: tunnelManagerError)
                self.finish(error: tunnelManagerError)
                return
            }

            let result = TunnelSettingsManager.update(searchTerm: .accountToken(token)) { tunnelSettings in
                self.modificationBlock(&tunnelSettings)
            }

            switch result {
            case .success(let newTunnelSettings):
                self.delegate?.operation(self, didSetTunnelSettings: newTunnelSettings)
                self.finish(error: nil)

            case .failure(let error):
                let tunnelManagerError = TunnelManager.Error.updateTunnelSettings(error)
                self.delegate?.operation(self, didFailToSetTunnelSettingsWithError: tunnelManagerError)
                self.finish(error: tunnelManagerError)
            }
        }
    }

    private func finish(error: TunnelManager.Error?) {
        completionHandler?(error)
        completionHandler = nil

        finish()
    }
}
