//
//  AccountDeletionViewModel.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-07-13.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation

class AccountDeletionViewModel: ObservableObject {
    enum State {
        case initial
        case working
        case failure(Error)
    }

    @Published var accountNumber: String
    @Published var enteredAccountNumberSuffix = ""
    @Published var state: State = .initial

    var interactor: AccountDeletionInteractor?
    var onConclusion: ((Bool) -> Void)?

    init(accountNumber: String, interactor: AccountDeletionInteractor? = nil, onConclusion: ((Bool) -> Void)? = nil) {
        self.accountNumber = accountNumber
        self.interactor = interactor
        self.onConclusion = onConclusion
    }

    var messageText: AttributedString {
        .init(NSAttributedString(
            markdownString: NSLocalizedString(
                """
                Are you sure you want to delete the account **\(accountNumber)**?
                """,
                comment: ""
            ),
            options: MarkdownStylingOptions(font: .preferredFont(forTextStyle: .body))
        ))
    }

    var statusText: String {
        switch state {
        case let .failure(error):
            return error.localizedDescription
        case .working:
            return NSLocalizedString("Deleting account...", comment: "")
        default: return ""
        }
    }

    var canDelete: Bool {
        !isWorking && enteredAccountNumberSuffix.count == 4 && accountNumber.suffix(4) == enteredAccountNumberSuffix
    }

    var isWorking: Bool {
        switch state {
        case .working: true
        default: false
        }
    }

    @MainActor func deleteButtonTapped() {
        guard let interactor else { return }
        switch interactor.validate(input: enteredAccountNumberSuffix) {
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
        guard let interactor else { return }
        state = .working
        Task { [weak self] in
            guard let self else { return }
            do {
                try await interactor.delete(accountNumber: accountNumber)
                self.state = State.initial
                self.onConclusion?(true)
            } catch {
                self.state = State.failure(error)
            }
        }
    }
}
