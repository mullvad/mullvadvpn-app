//
//  LoginViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 19/03/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import UIKit
import Logging

private let kMinimumAccountTokenLength = 10

enum AuthenticationMethod {
    case existingAccount, newAccount
}

enum LoginState {
    case `default`
    case authenticating(AuthenticationMethod)
    case failure(Account.Error)
    case success(AuthenticationMethod)
}

protocol LoginViewControllerDelegate: class {
    func loginViewController(_ controller: LoginViewController, loginWithAccountToken accountToken: String, completion: @escaping (Result<AccountResponse, Account.Error>) -> Void)
    func loginViewControllerLoginWithNewAccount(_ controller: LoginViewController, completion: @escaping (Result<AccountResponse, Account.Error>) -> Void)
    func loginViewControllerDidLogin(_ controller: LoginViewController)
}

class LoginViewController: UIViewController, RootContainment {

    private lazy var contentView: LoginContentView = {
        let view = LoginContentView(frame: self.view.bounds)
        view.translatesAutoresizingMaskIntoConstraints = false
        return view
    }()

    private lazy var accountInputAccessoryCancelButton: UIBarButtonItem = {
        return UIBarButtonItem(barButtonSystemItem: .cancel, target: self, action: #selector(cancelLogin))
    }()

    private lazy var accountInputAccessoryLoginButton: UIBarButtonItem = {
        return UIBarButtonItem(title: NSLocalizedString("Log in", comment: ""), style: .done, target: self, action: #selector(doLogin))
    }()

    private lazy var accountInputAccessoryToolbar: UIToolbar = {
        let toolbar = UIToolbar(frame: CGRect(x: 0, y: 0, width: 320, height: 44))
        toolbar.items = [
            self.accountInputAccessoryCancelButton,
            UIBarButtonItem(barButtonSystemItem: .flexibleSpace, target: nil, action: nil),
            self.accountInputAccessoryLoginButton
        ]
        toolbar.sizeToFit()
        return toolbar
    }()

    private let logger = Logger(label: "LoginViewController")

    private var loginState = LoginState.default {
        didSet {
            loginStateDidChange()
        }
    }

    weak var delegate: LoginViewControllerDelegate?

    override var preferredStatusBarStyle: UIStatusBarStyle {
        return .lightContent
    }

    var preferredHeaderBarStyle: HeaderBarStyle {
        return .transparent
    }

    var prefersHeaderBarHidden: Bool {
        return false
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        view.addSubview(contentView)
        NSLayoutConstraint.activate([
            contentView.topAnchor.constraint(equalTo: view.topAnchor),
            contentView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            contentView.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            contentView.bottomAnchor.constraint(equalTo: view.bottomAnchor),
        ])

        contentView.accountTextField.inputAccessoryView = self.accountInputAccessoryToolbar

        // The return key on iPad should behave the same way as "Log in" button in the toolbar
        if case .pad = UIDevice.current.userInterfaceIdiom {
            contentView.accountTextField.onReturnKey = { [weak self] _ in
                guard let self = self else { return true }

                if self.canBeginLogin() {
                    self.doLogin()
                    return true
                } else {
                    return false
                }
            }
        }

        updateDisplayedMessage()
        updateStatusIcon()
        updateKeyboardToolbar()

        let notificationCenter = NotificationCenter.default

        contentView.createAccountButton.addTarget(self, action: #selector(createNewAccount), for: .touchUpInside)

        notificationCenter.addObserver(self,
                                       selector: #selector(textDidChange(_:)),
                                       name: UITextField.textDidChangeNotification,
                                       object: contentView.accountTextField)
    }

    // MARK: - Public

    func reset() {
        loginState = .default
        contentView.accountTextField.autoformattingText = ""
        updateKeyboardToolbar()
    }

    // MARK: - UITextField notifications

    @objc func textDidChange(_ notification: Notification) {
        // Reset the text style as user start typing
        if case .failure = loginState {
            loginState = .default
        }

        // Enable the log in button in the keyboard toolbar
        updateKeyboardToolbar()
    }

    // MARK: - Actions

    @objc func cancelLogin() {
        view.endEditing(true)
    }

    @objc func doLogin() {
        let accountToken = contentView.accountTextField.parsedToken

        beginLogin(method: .existingAccount)
        self.delegate?.loginViewController(self, loginWithAccountToken: accountToken, completion: { [weak self] (result) in
            switch result {
            case .success:
                self?.endLogin(.success(.existingAccount))
            case .failure(let error):
                self?.endLogin(.failure(error))
            }
        })
    }

    @objc func createNewAccount() {
        beginLogin(method: .newAccount)

        contentView.accountTextField.autoformattingText = ""
        updateKeyboardToolbar()

        self.delegate?.loginViewControllerLoginWithNewAccount(self, completion: { [weak self] (result) in
            switch result {
            case .success(let response):
                self?.contentView.accountTextField.autoformattingText = response.token
                self?.endLogin(.success(.newAccount))
            case .failure(let error):
                self?.endLogin(.failure(error))
            }
        })
    }

    // MARK: - Private

    private func loginStateDidChange() {
        contentView.accountInputGroup.loginState = loginState

        // Keep the settings button disabled to prevent user from going to settings while
        // authentication or during the delay after the successful login and transition to the main
        // controller.
        switch loginState {
        case .authenticating:
            contentView.activityIndicator.startAnimating()
            contentView.createAccountButton.isEnabled = false

        case .success:
            break

        case .default, .failure:
            contentView.createAccountButton.isEnabled = true
            contentView.activityIndicator.stopAnimating()
        }

        updateDisplayedMessage()
        updateStatusIcon()
    }

    private func updateStatusIcon() {
        switch loginState {
        case .failure:
            contentView.setStatusImage(style: .failure, visible: true, animated: true)

        case .success:
            contentView.setStatusImage(style: .success, visible: true, animated: true)

        case .default, .authenticating:
            contentView.setStatusImage(style: nil, visible: false, animated: true)
        }
    }

    private func beginLogin(method: AuthenticationMethod) {
        loginState = .authenticating(method)

        view.endEditing(true)
    }

    private func endLogin(_ nextLoginState: LoginState) {
        let oldLoginState = loginState

        loginState = nextLoginState

        if case .authenticating(.existingAccount) = oldLoginState,
            case .failure = loginState {
            contentView.accountTextField.becomeFirstResponder()
        } else if case .success = loginState {
            // Navigate to the main view after 1s delay
            DispatchQueue.main.asyncAfter(deadline: .now() + .seconds(1)) {
                self.delegate?.loginViewControllerDidLogin(self)
            }
        }
    }

    private func updateDisplayedMessage() {
        contentView.titleLabel.text = loginState.localizedTitle
        contentView.messageLabel.text = loginState.localizedMessage
    }

    private func updateKeyboardToolbar() {
        accountInputAccessoryLoginButton.isEnabled = canBeginLogin()
        contentView.accountTextField.enableReturnKey = canBeginLogin()
    }

    private func canBeginLogin() -> Bool {
        let accountTokenLength = contentView.accountTextField.parsedToken.count
        return accountTokenLength >= kMinimumAccountTokenLength
    }
}

/// Private extension that brings localizable messages displayed in the Login view controller
private extension LoginState {
    var localizedTitle: String {
        switch self {
        case .default:
            return NSLocalizedString("Login", comment: "")

        case .authenticating:
            return NSLocalizedString("Logging in...", comment: "")

        case .failure:
            return NSLocalizedString("Login failed", comment: "")

        case .success:
            return NSLocalizedString("Logged in", comment: "")
        }
    }

    var localizedMessage: String {
        switch self {
        case .default:
            return NSLocalizedString("Enter your account number", comment: "")

        case .authenticating(let method):
            switch method {
            case .existingAccount:
                return NSLocalizedString("Checking account number", comment: "")
            case .newAccount:
                return NSLocalizedString("Creating new account", comment: "")
            }

        case .failure(let error):
            switch error {
            case .createAccount(let rpcError), .verifyAccount(let rpcError):
                return rpcError.errorChainDescription ?? ""
            case .tunnelConfiguration(let error):
                if case .pushWireguardKey(let pushError) = error {
                    switch pushError {
                    case .network(let urlError):
                        return String(
                            format: NSLocalizedString("Network error: %@", comment: ""),
                            urlError.localizedDescription
                        )

                    case .server(let serverError):
                        var message = serverError.errorDescription ?? NSLocalizedString("Unknown server error.", comment: "")

                        if let recoverySuggestion = serverError.recoverySuggestion {
                            message.append("\n\(recoverySuggestion)")
                        }

                        return message

                    case .encodePayload, .decodeErrorResponse, .decodeSuccessResponse:
                        return NSLocalizedString("Internal error", comment: "")
                    }
                } else {
                    return NSLocalizedString("Internal error", comment: "")
                }
            }

        case .success(let method):
            switch method {
            case .existingAccount:
                return NSLocalizedString("Correct account number", comment: "")
            case .newAccount:
                return NSLocalizedString("Account created", comment: "")
            }
        }
    }
}
