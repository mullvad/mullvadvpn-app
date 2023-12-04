//
//  AccountDeletionInteractor.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-07-13.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes

enum AccountDeletionError: LocalizedError {
    case invalidInput

    var errorDescription: String? {
        switch self {
        case .invalidInput:
            return NSLocalizedString(
                "INVALID_ACCOUNT_NUMBER",
                tableName: "Account",
                value: "Last four digits of the account number are incorrect",
                comment: ""
            )
        }
    }
}

class AccountDeletionInteractor {
    private let tunnelManager: TunnelManager
    var viewModel: AccountDeletionViewModel {
        AccountDeletionViewModel(
            accountNumber: tunnelManager.deviceState.accountData?.number.formattedAccountNumber ?? ""
        )
    }

    init(tunnelManager: TunnelManager) {
        self.tunnelManager = tunnelManager
    }

    func validate(input: String) -> Result<String, Error> {
        if let accountNumber = tunnelManager.deviceState.accountData?.number,
           let fourLastDigits = accountNumber.split(every: 4).last,
           fourLastDigits == input {
            return .success(accountNumber)
        } else {
            return .failure(AccountDeletionError.invalidInput)
        }
    }

    func delete(accountNumber: String, completionHandler: @escaping (Error?) -> Void) {
        tunnelManager.deleteAccount(accountNumber: accountNumber, completion: completionHandler)
    }
}
