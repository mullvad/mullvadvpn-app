//
//  RevokedDeviceViewModel.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-10-02.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadMockData
import SwiftUICore

class RevokedDeviceViewModel: ObservableObject {
    private let interactor: RevokedDeviceInteractor
    @Published var tunnelState: TunnelState

    var onLogout: (() -> Void)?

    init(interactor: RevokedDeviceInteractor) {
        self.interactor = interactor
    }

    // for SwiftUI previews
    init(tunnelState: TunnelState) {

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
