// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import SwiftUI

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

    let onDeleteAccount: () async throws -> Void

    var onConclusion: ((Bool) -> Void)?

    var accountNumberSuffix: Substring {
        accountNumber.suffix(4)
    }

    init(
        accountNumber: String,
        onDeleteAccount: @escaping () async throws -> Void,
        onConclusion: ((Bool) -> Void)? = nil
    ) {
        self.onDeleteAccount = onDeleteAccount
        self.accountNumber = accountNumber
        self.onConclusion = onConclusion
    }

    var messageText: LocalizedStringKey {
        var attributedAccountNumber: AttributedString {
            return
                (try? AttributedString(
                    markdown: "**\(accountNumber.formattedAccountNumber)**",
                    options: AttributedString.MarkdownParsingOptions(interpretedSyntax: .inlineOnlyPreservingWhitespace)
                )) ?? AttributedString(accountNumber)
        }
        return LocalizedStringKey("Are you sure you want to delete the account \(attributedAccountNumber)?")
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

    func validate(input: String) -> Result<Void, Error> {
        if let fourLastDigits = accountNumber.split(every: 4).last,
            fourLastDigits == input
        {
            return .success(())
        } else {
            return .failure(Error.invalidInput)
        }
    }

    @discardableResult
    @MainActor func deleteButtonTapped() async -> Bool {
        switch validate(input: enteredAccountNumberSuffix) {
        case .success:
            return await doDelete()
        case let .failure(error):
            state = .failure(error)
            return false
        }
    }

    func cancelButtonTapped() {
        self.onConclusion?(false)
    }

    @MainActor func doDelete() async -> Bool {
        state = .working
        do {
            try await onDeleteAccount()
            state = State.initial
            onConclusion?(true)
            return true
        } catch {
            state = State.failure(error)
            return false
        }
    }
}
