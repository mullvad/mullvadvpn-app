//
//  AccountDeletionInteractor.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-07-13.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes

enum AccountDeletionError: LocalizedError {
    case invalidInput

    var errorDescription: String? {
        switch self {
        case .invalidInput:
            return NSLocalizedString("Last four digits of the account number are incorrect", comment: "")
        }
    }
}

final class AccountDeletionInteractor: Sendable {
    private let tunnelManager: TunnelManager

    func makeViewModel(onConclusion: @escaping ((Bool) -> Void)) -> AccountDeletionViewModel {
        .init(
            accountNumber: tunnelManager.deviceState.accountData?.number.formattedAccountNumber ?? "",
            interactor: self,
            onConclusion: onConclusion
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

    func delete(accountNumber: String) async throws {
        try await tunnelManager.deleteAccount(accountNumber: accountNumber)
    }
}
