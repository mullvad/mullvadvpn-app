//
//  ReconnectTunnelOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 10/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes
import Operations
import RelayCache
import RelaySelector

class ReconnectTunnelOperation: ResultOperation<Void> {
    private let interactor: TunnelInteractor
    private let selectNewRelay: Bool
    private let reconnectionDelay: Int?
    private var task: Cancellable?

    init(
        dispatchQueue: DispatchQueue,
        interactor: TunnelInteractor,
        selectNewRelay: Bool,
        reconnectionDelay: Int?
    ) {
        self.interactor = interactor
        self.selectNewRelay = selectNewRelay
        self.reconnectionDelay = reconnectionDelay

        super.init(dispatchQueue: dispatchQueue)
    }

    override func main() {
        guard let tunnel = interactor.tunnel else {
            finish(result: .failure(UnsetTunnelError()))
            return
        }

        do {
            let selectorResult = selectNewRelay ? try interactor.selectRelay() : nil

            task = tunnel
                .reconnectTunnel(
                    relaySelectorResult: selectorResult,
                    reconnectionDelay: reconnectionDelay
                ) { [weak self] result in
                    self?.finish(result: result)
                }
        } catch {
            finish(result: .failure(error))
        }
    }

    override func operationDidCancel() {
        task?.cancel()
        task = nil
    }
}
