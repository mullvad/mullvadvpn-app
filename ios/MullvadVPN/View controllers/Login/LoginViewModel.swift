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

@MainActor
protocol LoginViewModel: ObservableObject {
    var accountNumber: String { get set }
    var storedAccountNumber: [String] { get set }
    var showButtons: Bool { get }
    var enableLoginButton: Bool { get }
    var statusActivityState: StatusActivityView.State { get }
    var textFieldBorderStyle: Binding<BorderStyle> { get set }
    var errorMessage: Binding<MessageView.Message?> { get set }
    var textFieldIsDisabled: Bool { get }
    var showAccessMethodInvalidView: Bool { get set }
    var navigateToAccessMethods: (() -> Void)? { get }

    func login()
    func createAccount()
}

protocol LoginViewModelProviding {
    func setExistingAccount(accountNumber: String) async throws -> StoredAccountData
    func setNewAccount() async throws -> StoredAccountData
}
extension TunnelManager: LoginViewModelProviding {}

@MainActor
class LoginViewModelImpl: LoginViewModel {
    enum LoginState {
        case `default`
        case authenticating(LoginAction)
        case failure(LoginAction, Error)
        case success(LoginAction)
    }

    enum LoginAction: Equatable {
        case login(String)
        case createAccount
    }

    @Published var accountNumber: String = "" {
        willSet {
            switch loginState {
            case .failure, .authenticating, .success:
                if accountNumber != newValue {
                    loginState = .default
                }
            case .default:
                break
            }
        }
    }

    @Published var storedAccountNumber: [String] = []
    @Published var loginState: LoginState
    @Published var showAccessMethodInvalidView: Bool = false

    var didFinishLogin: ((LoginAction, Error?) -> Void)?
    var navigateToAccessMethods: (() -> Void)?

    private let logger = Logger(label: "LoginViewController")
    private let minimumAccountTokenLength = 16
    private let interactor: LoginInteractor

    private var nonTokenizedAccountNumber: String {
        accountNumber.replacingOccurrences(of: " ", with: "")
    }

    init(
        interactor: LoginInteractor,
        loginState: LoginState = .default
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
                _ = try await (minimumDelay, createAccount)

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

extension LoginViewModelImpl {
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
        case .authenticating:
            true
        case .default, .failure, .success:
            false
        }
    }

    var textFieldBorderStyle: Binding<BorderStyle> {
        get {
            switch loginState {
            case .failure:
                .constant(.error)
            case .success:
                .constant(.normal)
            case .authenticating:
                .constant(.none)
            case .default:
                .constant(.focused)
            }
        }
        set {}
    }

    var errorMessage: Binding<MessageView.Message?> {
        get {
            switch loginState {
            case .failure(_, let error):
                .constant(
                    .init(
                        text: (error as? DisplayError)?.displayErrorDescription ?? error.localizedDescription,
                        appearance: .error
                    )
                )
            case .authenticating, .default, .success:
                .constant(nil)
            }
        }
        set {}
    }
}
