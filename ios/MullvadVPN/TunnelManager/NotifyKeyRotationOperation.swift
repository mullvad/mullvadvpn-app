//
//  NotifyKeyRotationOperation.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-03-28.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import Operations

class NotifyKeyRotationOperation: ResultOperation<Void> {
    private let interactor: TunnelInteractor
    private var task: Cancellable?

    init(
        dispatchQueue: DispatchQueue,
        interactor: TunnelInteractor
    ) {
        self.interactor = interactor

        super.init(dispatchQueue: dispatchQueue)
    }

    override func main() {
        guard let tunnel = interactor.tunnel else {
            finish(result: .failure(UnsetTunnelError()))
            return
        }

        task = tunnel.notifyKeyRotation { [weak self] result in
            self?.finish(result: result)
        }
    }

    override func operationDidCancel() {
        task?.cancel()
        task = nil
    }
}
