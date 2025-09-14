//
//  AccountDeletionViewModel.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-07-13.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import SwiftUI

protocol AccountDeletionBackEnd {
    var accountNumber: String? { get }

    func deleteAccount(accountNumber: String) async throws
}

struct TunnelManagerAccountDeletionBackEnd: AccountDeletionBackEnd {
    let tunnelManager: TunnelManager

    var accountNumber: String? {
        tunnelManager.deviceState.accountData?.number
    }

    func deleteAccount(accountNumber: String) async throws {
        try await tunnelManager.deleteAccount(accountNumber: accountNumber)
    }
}

struct MockAccountDeletionBackEnd: AccountDeletionBackEnd {
    let accountNumber: String?

    func deleteAccount(accountNumber: String) async throws {}
}

class AccountDeletionViewModel: ObservableObject {
    enum State {
        case initial
        case working
        case failure(Swift.Error)
    }

    enum Error: LocalizedError {
        case invalidInput

        var errorDescription: String? {
            switch self {
            case .invalidInput:
                return NSLocalizedString("Last four digits of the account number are incorrect", comment: "")
            }
        }
    }

    @Published var accountNumber: String
    @Published var enteredAccountNumberSuffix = ""
    @Published var state: State = .initial

    private let backEnd: AccountDeletionBackEnd

    var onConclusion: ((Bool) -> Void)?

    var tunnelManagerAccountNumber: String {
        backEnd.accountNumber ?? ""
    }

    var accountNumberSuffix: Substring {
        accountNumber.suffix(4)
    }

    init(tunnelManager: TunnelManager, onConclusion: ((Bool) -> Void)? = nil) {
        self.backEnd = TunnelManagerAccountDeletionBackEnd(tunnelManager: tunnelManager)
        self.accountNumber = tunnelManager.deviceState.accountData?.number.formattedAccountNumber ?? ""
        self.onConclusion = onConclusion
    }

    // for SwiftUI previews
    init(mockAccountNumber: String?) {
        self.backEnd = MockAccountDeletionBackEnd(accountNumber: mockAccountNumber)
        self.accountNumber = mockAccountNumber ?? ""
        self.onConclusion = nil
    }

    var messageText: AttributedString {
        .fromMarkdown(
            """
            Are you sure you want to delete the account **\(accountNumber)**?
            """
        )
    }

    var statusText: LocalizedStringKey? {
        switch state {
        case let .failure(error):
            LocalizedStringKey(error.localizedDescription)
        case .working:
            LocalizedStringKey("Deleting account...")
        default: nil
        }
    }

    var canDelete: Bool {
        !isWorking && enteredAccountNumberSuffix.count == 4 && accountNumberSuffix == enteredAccountNumberSuffix
    }

    var isWorking: Bool {
        switch state {
        case .working: true
        default: false
        }
    }

    func validate(input: String) -> Result<String, Error> {
        if let deviceAccountNumber = backEnd.accountNumber,
            let fourLastDigits = deviceAccountNumber.split(every: 4).last,
            fourLastDigits == input
        {
            return .success(deviceAccountNumber)
        } else {
            return .failure(Error.invalidInput)
        }
    }

    @MainActor func deleteButtonTapped() {
        switch validate(input: enteredAccountNumberSuffix) {
        case let .success(accountNumber):
            doDelete(accountNumber: accountNumber)
        case let .failure(error):
            state = .failure(error)
        }
    }

    func cancelButtonTapped() {
        self.onConclusion?(false)
    }

    @MainActor func doDelete(accountNumber: String) {
        state = .working
        Task { [weak self] in
            guard let self else { return }
            do {
                try await backEnd.deleteAccount(accountNumber: accountNumber)
                self.state = State.initial
                self.onConclusion?(true)
            } catch {
                self.state = State.failure(error)
            }
        }
    }
}
