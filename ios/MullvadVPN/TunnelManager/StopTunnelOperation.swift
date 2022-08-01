//
//  StopTunnelOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 15/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

class StopTunnelOperation: ResultOperation<(), Error> {
    private let interactor: TunnelInteractor

    init(
        dispatchQueue: DispatchQueue,
        interactor: TunnelInteractor,
        completionHandler: @escaping CompletionHandler
    )
    {
        self.interactor = interactor

        super.init(
            dispatchQueue: dispatchQueue,
            completionQueue: dispatchQueue,
            completionHandler: completionHandler
        )
    }

    override func main() {
        switch interactor.tunnelStatus.state {
        case .disconnecting(.reconnect):
            interactor.updateTunnelState(.disconnecting(.nothing))

            finish(completion: .success(()))

        case .connected, .connecting, .reconnecting:
            guard let tunnel = interactor.tunnel else {
                finish(completion: .failure(UnsetTunnelError()))
                return
            }

            // Disable on-demand when stopping the tunnel to prevent it from coming back up
            tunnel.isOnDemandEnabled = false

            tunnel.saveToPreferences { error in
                self.dispatchQueue.async {
                    if let error = error {
                        self.finish(completion: .failure(error))
                    } else {
                        tunnel.stop()
                        self.finish(completion: .success(()))
                    }
                }
            }

        default:
            finish(completion: .success(()))
        }
    }
}
