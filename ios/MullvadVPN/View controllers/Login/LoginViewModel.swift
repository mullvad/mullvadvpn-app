//
//  LoginViewModel.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-07-28.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import MullvadSettings
import MullvadTypes
import SwiftUI

protocol LoginViewModelProviding {
    func setExistingAccount(accountNumber: String) async throws -> StoredAccountData
    func setNewAccount() async throws -> StoredAccountData
}
extension TunnelManager: LoginViewModelProviding {}

@MainActor
@Observable class LoginViewModel {
    enum State {
        case `default`
        case authenticating(Action)
        case failure(Action, Error)
        case success(Action)
    }

    enum Action: Equatable {
        case login(String)
        case createAccount
    }

    var accountNumber: String = "" {
        willSet {
            switch loginState {
            case .failure, .authenticating:
                if accountNumber != newValue {
                    loginState = .default
                }
            case .default, .success:
                break
            }
        }
    }

    var storedAccountNumber: [String] = [] {
        didSet {
            if storedAccountNumber.isEmpty {
                interactor.removeLastUsedAccount()
            }
        }
    }

    var loginState: State
    var showAccessMethodInvalidView: Bool = false

    var didFinishLogin: ((Action, Error?) -> Void)?
    var navigateToAccessMethods: (() -> Void)?

    private let logger = Logger(label: "LoginViewController")
    private let minimumAccountTokenLength = 11
    private let interactor: LoginInteractor

    private var nonTokenizedAccountNumber: String {
        accountNumber.replacingOccurrences(of: " ", with: "")
    }

    init(
        interactor: LoginInteractor,
        loginState: State = .default
    ) {
        self.interactor = interactor
        self.loginState = loginState

        let lastUsedAccount = interactor.getLastUsedAccount()
        let parsedLastUsedAccount = lastUsedAccount?.split(every: 4).joined(separator: " ")
        if let parsedLastUsedAccount {
            storedAccountNumber = [parsedLastUsedAccount]
        }
    }

    func login() {
        Task { [weak self] in
            guard let self else { return }

            let accountNumber = nonTokenizedAccountNumber
            loginState = .authenticating(.login(accountNumber))

            do {
                async let minimumDelay: Void = Task.sleep(for: .seconds(1.5))
                async let login: Void = interactor.setAccount(accountNumber: accountNumber)
                _ = try await (minimumDelay, login)

                loginState = .success(.login(accountNumber))
                didFinishLogin?(.login(accountNumber), nil)
            } catch {
                loginState = .failure(.login(accountNumber), error)
                didFinishLogin?(.login(accountNumber), error)
            }
        }
    }

    func createAccount() {
        Task { [weak self] in
            guard let self else { return }

            loginState = .authenticating(.createAccount)

            do {
                async let minimumDelay: Void = Task.sleep(for: .seconds(1.5))
                async let createAccount: String = interactor.createAccount()
                let (_, accountNumber) = try await (minimumDelay, createAccount)

                self.accountNumber = accountNumber

                loginState = .success(.createAccount)
                didFinishLogin?(.createAccount, nil)
            } catch {
                loginState = .failure(.createAccount, error)
                didFinishLogin?(.createAccount, error)
            }
        }
    }

    private var satisfiesMinimumTokenLengthRequirement: Bool {
        nonTokenizedAccountNumber.count >= minimumAccountTokenLength
    }
}

// MARK: Computed state

extension LoginViewModel {
    var showButtons: Bool {
        return switch loginState {
        case .default, .failure:
            true
        case .authenticating, .success:
            false
        }
    }

    var enableLoginButton: Bool {
        return switch loginState {
        case .default, .failure:
            satisfiesMinimumTokenLengthRequirement
        case .authenticating, .success:
            false
        }
    }

    var statusActivityState: StatusActivityView.State {
        switch loginState {
        case .failure:
            .failure
        case .success:
            .success
        case .authenticating:
            .activity
        case .default:
            .hidden
        }
    }

    var textFieldIsDisabled: Bool {
        switch loginState {
        case .authenticating, .success:
            true
        case .default, .failure:
            false
        }
    }

    var textFieldBorderStyle: BorderStyle {
        switch loginState {
        case .failure:
            .error
        case .success:
            .normal
        case .authenticating:
            .none
        case .default:
            .focused
        }
    }

    var errorMessage: MessageView.Message? {
        switch loginState {
        case .failure(_, let error):
            .init(
                text: (error as? DisplayError)?.displayErrorDescription ?? error.localizedDescription,
                appearance: .error
            )
        case .authenticating, .default, .success:
            nil
        }
    }
}
