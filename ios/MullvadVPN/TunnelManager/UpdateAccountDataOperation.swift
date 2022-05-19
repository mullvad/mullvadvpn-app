//
//  UpdateAccountDataOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 12/05/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Logging

class UpdateAccountDataOperation: ResultOperation<Void, TunnelManager.Error> {
    private let logger = Logger(label: "UpdateAccountDataOperation")
    private let state: TunnelManager.State
    private let accountsProxy: REST.AccountsProxy
    private var task: Cancellable?

    init(
        dispatchQueue: DispatchQueue,
        state: TunnelManager.State,
        accountsProxy: REST.AccountsProxy
    )
    {
        self.state = state
        self.accountsProxy = accountsProxy

        super.init(dispatchQueue: dispatchQueue)
    }

    override func main() {
        guard let tunnelSettings = state.tunnelSettings else {
            finish(completion: .failure(.unsetAccount))
            return
        }

        task = accountsProxy.getAccountData(
            accountNumber: tunnelSettings.account.number,
            retryStrategy: .default
        ) { completion in
            self.dispatchQueue.async {
                self.didReceiveAccountData(
                    tunnelSettings: tunnelSettings,
                    completion: completion
                )
            }
        }
    }

    override func operationDidCancel() {
        task?.cancel()
        task = nil
    }

    private func didReceiveAccountData(
        tunnelSettings: TunnelSettingsV2,
        completion: OperationCompletion<REST.AccountData, REST.Error>
    )
    {
        let mappedCompletion = completion.mapError { error -> TunnelManager.Error in
            self.logger.error(
                chainedError: error,
                message: "Failed to fetch account expiry."
            )
            return .getAccountData(error)
        }

        guard let accountData = mappedCompletion.value else {
            finish(completion: mappedCompletion.assertNoSuccess())
            return
        }

        do {
            var newTunnelSettings = tunnelSettings
            newTunnelSettings.account.expiry = accountData.expiry
            try SettingsManager.writeSettings(newTunnelSettings)

            finish(completion: .success(()))
        } catch {
            self.logger.error(
                chainedError: AnyChainedError(error),
                message: "Failed to save account data."
            )

            finish(completion: .failure(.writeSettings(error)))
        }
    }
}
