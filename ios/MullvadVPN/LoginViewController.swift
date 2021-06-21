//
//  LoginViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 19/03/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import UIKit
import Logging

enum AuthenticationMethod {
    case existingAccount, newAccount
}

enum LoginState {
    case `default`
    case authenticating(AuthenticationMethod)
    case failure(Account.Error)
    case success(AuthenticationMethod)
}

protocol LoginViewControllerDelegate: AnyObject {
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
        let barButtonItem = UIBarButtonItem(
            title: NSLocalizedString(
                "LOGIN_ACCESSORY_TOOLBAR_BUTTON_TITLE",
                tableName: "Login",
                comment: "Title for 'Log in' button displayed in toolbar above keyboard on iPhone."
            ),
            style: .done,
            target: self,
            action: #selector(doLogin)
        )
        barButtonItem.accessibilityIdentifier = "LoginBarButtonItem"

        return barButtonItem
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

    var preferredHeaderBarPresentation: HeaderBarPresentation {
        return HeaderBarPresentation(style: .transparent, showsDivider: false)
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

        contentView.accountInputGroup.onSendButton = { [weak self] _ in
            guard let self = self else { return }

            if self.canBeginLogin() {
                self.doLogin()
            }
        }

        contentView.accountInputGroup.setOnReturnKey { [weak self] _ in
            guard let self = self else { return true }

            if self.canBeginLogin() {
                self.doLogin()
                return true
            } else {
                return false
            }
        }

        // There is no need to set the input accessory toolbar on iPad since it has a dedicated
        // button to dismiss the keyboard.
        if case .phone = UIDevice.current.userInterfaceIdiom {
            contentView.accountInputGroup.textField.inputAccessoryView = self.accountInputAccessoryToolbar
        } else {
            contentView.accountInputGroup.textField.inputAccessoryView = nil
        }

        updateDisplayedMessage()
        updateStatusIcon()
        updateKeyboardToolbar()

        let notificationCenter = NotificationCenter.default

        contentView.createAccountButton.addTarget(self, action: #selector(createNewAccount), for: .touchUpInside)

        notificationCenter.addObserver(self,
                                       selector: #selector(textDidChange(_:)),
                                       name: UITextField.textDidChangeNotification,
                                       object: contentView.accountInputGroup.textField)
    }

    override var disablesAutomaticKeyboardDismissal: Bool {
        // Allow dismissing the keyboard in .formSheet presentation style
        return false
    }

    // MARK: - Public

    func reset() {
        loginState = .default
        contentView.accountInputGroup.clearToken()
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
        let accountToken = contentView.accountInputGroup.parsedToken

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

        contentView.accountInputGroup.clearToken()
        updateKeyboardToolbar()

        self.delegate?.loginViewControllerLoginWithNewAccount(self, completion: { [weak self] (result) in
            switch result {
            case .success(let response):
                self?.contentView.accountInputGroup.setToken(response.token)
                self?.endLogin(.success(.newAccount))
            case .failure(let error):
                self?.endLogin(.failure(error))
            }
        })
    }

    // MARK: - Private

    private func loginStateDidChange() {
        contentView.accountInputGroup.setLoginState(loginState, animated: true)

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

        if case .authenticating(.existingAccount) = oldLoginState, case .failure = loginState {
            contentView.accountInputGroup.textField.becomeFirstResponder()
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
    }

    private func canBeginLogin() -> Bool {
        return contentView.accountInputGroup.satisfiesMinimumTokenLengthRequirement
    }
}

/// Private extension that brings localizable messages displayed in the Login view controller
private extension LoginState {
    var localizedTitle: String {
        switch self {
        case .default:
            return NSLocalizedString("HEADING_TITLE_DEFAULT", tableName: "Login", comment: "Default login prompt heading.")

        case .authenticating:
            return NSLocalizedString("HEADING_TITLE_AUTHENTICATING", tableName: "Login", comment: "Heading displayed during authentication.")

        case .failure:
            return NSLocalizedString("HEADING_TITLE_FAILURE", tableName: "Login", comment: "Heading displayed upon failure to authenticate.")

        case .success:
            return NSLocalizedString("HEADING_TITLE_SUCCESS", tableName: "Login", comment: "Heading displayed upon successful authentication.")
        }
    }

    var localizedMessage: String {
        switch self {
        case .default:
            return NSLocalizedString(
                "SUBHEAD_TITLE_DEFAULT",
                tableName: "Login",
                comment: "Default login prompt subhead."
            )

        case .authenticating(let method):
            switch method {
            case .existingAccount:
                return NSLocalizedString(
                    "SUBHEAD_TITLE_AUTHENTICATING",
                    tableName: "Login",
                    comment: "Subhead displayed during authentication."
                )
            case .newAccount:
                return NSLocalizedString(
                    "SUBHEAD_TITLE_CREATING_ACCOUNT",
                    tableName: "Login",
                    comment: "Subhead displayed when creating new account."
                )
            }

        case .failure(let error):
            let localizedUnknownInternalError = NSLocalizedString(
                "SUBHEAD_TITLE_INTERNAL_ERROR",
                tableName: "Login",
                comment: "Subhead displayed in the event of internal error."
            )

            switch error {
            case .createAccount(let rpcError), .verifyAccount(let rpcError):
                return rpcError.errorChainDescription ?? ""
            case .tunnelConfiguration(let error):
                if case .pushWireguardKey(let pushError) = error {
                    switch pushError {
                    case .network(let urlError):
                        return String(
                            format: NSLocalizedString(
                                "SUBHEAD_TITLE_NETWORK_ERROR_FORMAT",
                                tableName: "Login",
                                value: "Network error: %@",
                                comment: "Subhead displayed in the event of network error. Use %@ placeholder to place localized text describing network failure."
                            ),
                            urlError.localizedDescription
                        )

                    case .server(let serverError):
                        var message = serverError.errorDescription ?? NSLocalizedString(
                            "SUBHEAD_TITLE_UNKNOWN_SERVER_ERROR",
                            tableName: "Login",
                            comment: "Subhead displayed in the event of unknown server error."
                        )

                        if let recoverySuggestion = serverError.recoverySuggestion {
                            message.append("\n\(recoverySuggestion)")
                        }

                        return message

                    case .encodePayload, .decodeErrorResponse, .decodeSuccessResponse:
                        return localizedUnknownInternalError
                    }
                } else {
                    return localizedUnknownInternalError
                }
            }

        case .success(let method):
            switch method {
            case .existingAccount:
                return NSLocalizedString(
                    "SUBHEAD_TITLE_SUCCESS",
                    tableName: "Login",
                    comment: "Subhead displayed upon successful authentication using existing account token."
                )
            case .newAccount:
                return NSLocalizedString(
                    "SUBHEAD_TITLE_CREATED_ACCOUNT",
                    tableName: "Login",
                    comment: "Subhead displayed upon successful authentication with newly created account token."
                )
            }
        }
    }
}
