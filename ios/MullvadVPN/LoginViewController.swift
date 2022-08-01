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
    case failure(Error)
    case success(AuthenticationMethod)
}

protocol LoginViewControllerDelegate: AnyObject {
    func loginViewController(
        _ controller: LoginViewController,
        loginWithAccountToken accountToken: String,
        completion: @escaping (OperationCompletion<StoredAccountData?, Error>) -> Void
    )

    func loginViewControllerLoginWithNewAccount(
        _ controller: LoginViewController,
        completion: @escaping (OperationCompletion<StoredAccountData?, Error>) -> Void
    )

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
                value: "Log in",
                comment: ""
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

    private var canBeginLogin: Bool {
        return contentView.accountInputGroup.satisfiesMinimumTokenLengthRequirement
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
        updateLastUsedAccount()

        contentView.accountInputGroup.delegate = self

        contentView.accountInputGroup.setOnReturnKey { [weak self] _ in
            guard let self = self else { return true }

            return self.attemptLogin()
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

    override func traitCollectionDidChange(_ previousTraitCollection: UITraitCollection?) {
        if traitCollection.userInterfaceIdiom != previousTraitCollection?.userInterfaceIdiom {
            updateCreateButtonEnabled()
        }
    }

    // MARK: - Public

    func reset() {
        contentView.accountInputGroup.clearAccount()
        loginState = .default
        updateKeyboardToolbar()
        updateLastUsedAccount()
    }

    // MARK: - UITextField notifications

    @objc func textDidChange(_ notification: Notification) {
        // Reset the text style as user start typing
        if case .failure = loginState {
            loginState = .default
        }

        // Enable the log in button in the keyboard toolbar.
        updateKeyboardToolbar()

        // Update "create account" button state.
        updateCreateButtonEnabled()
    }

    // MARK: - Actions

    @objc func cancelLogin() {
        view.endEditing(true)
    }

    @objc func doLogin() {
        let accountToken = contentView.accountInputGroup.parsedToken

        beginLogin(method: .existingAccount)
        self.delegate?.loginViewController(self, loginWithAccountToken: accountToken, completion: { [weak self] completion in
            switch completion {
            case .success:
                self?.endLogin(.success(.existingAccount))
            case .failure(let error):
                self?.endLogin(.failure(error))
            case .cancelled:
                self?.endLogin(.default)
            }
        })
    }

    @objc func createNewAccount() {
        beginLogin(method: .newAccount)

        contentView.accountInputGroup.clearAccount()
        updateKeyboardToolbar()

        self.delegate?.loginViewControllerLoginWithNewAccount(self, completion: { [weak self] completion in
            switch completion {
            case .success(let accountData):
                self?.contentView.accountInputGroup.setAccount(accountData?.number ?? "")
                self?.endLogin(.success(.newAccount))
            case .failure(let error):
                self?.endLogin(.failure(error))
            case .cancelled:
                self?.endLogin(.default)
            }
        })
    }

    // MARK: - Private

    private func updateLastUsedAccount() {
        do {
            let accountNumber = try SettingsManager.getLastUsedAccount()

            contentView.accountInputGroup.setLastUsedAccount(accountNumber, animated: false)
        } catch {
            logger.error(chainedError: AnyChainedError(error),
                         message: "Failed to update last used account.")
        }
    }

    private func loginStateDidChange() {
        contentView.accountInputGroup.setLoginState(loginState, animated: true)

        switch loginState {
        case .authenticating:
            contentView.activityIndicator.startAnimating()

        case .success, .default, .failure:
            contentView.activityIndicator.stopAnimating()
        }

        updateDisplayedMessage()
        updateStatusIcon()
        updateCreateButtonEnabled()
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
        accountInputAccessoryLoginButton.isEnabled = canBeginLogin
    }

    private func updateCreateButtonEnabled() {
        let isEnabled: Bool

        switch loginState {
        case .failure, .default:
            // Disable "Create account" button on iPad as user types in the account token,
            // however leave it enabled on iPhone to avoid confusion to why it's being disabled
            // since it's likely overlayed by keyboard.
            if case .pad = self.traitCollection.userInterfaceIdiom {
                isEnabled = contentView.accountInputGroup.textField.text?.isEmpty ?? true
            } else {
                isEnabled = true
            }

        case .success, .authenticating:
            isEnabled = false
        }

        contentView.createAccountButton.isEnabled = isEnabled
    }

    @discardableResult private func attemptLogin() -> Bool {
        if canBeginLogin {
            doLogin()
            return true
        } else {
            return false
        }
    }
}

/// Private extension that brings localizable messages displayed in the Login view controller
private extension LoginState {
    var localizedTitle: String {
        switch self {
        case .default:
            return NSLocalizedString(
                "HEADING_TITLE_DEFAULT",
                tableName: "Login",
                value: "Login",
                comment: ""
            )

        case .authenticating:
            return NSLocalizedString(
                "HEADING_TITLE_AUTHENTICATING",
                tableName: "Login",
                value: "Logging in...",
                comment: ""
            )

        case .failure:
            return NSLocalizedString(
                "HEADING_TITLE_FAILURE",
                tableName: "Login",
                value: "Login failed",
                comment: ""
            )

        case .success:
            return NSLocalizedString(
                "HEADING_TITLE_SUCCESS",
                tableName: "Login",
                value: "Logged in",
                comment: ""
            )
        }
    }

    var localizedMessage: String {
        switch self {
        case .default:
            return NSLocalizedString(
                "SUBHEAD_TITLE_DEFAULT",
                tableName: "Login",
                value: "Enter your account number",
                comment: ""
            )

        case .authenticating(let method):
            switch method {
            case .existingAccount:
                return NSLocalizedString(
                    "SUBHEAD_TITLE_AUTHENTICATING",
                    tableName: "Login",
                    value: "Checking account number",
                    comment: ""
                )
            case .newAccount:
                return NSLocalizedString(
                    "SUBHEAD_TITLE_CREATING_ACCOUNT",
                    tableName: "Login",
                    value: "Creating new account",
                    comment: ""
                )
            }

        case .failure(let error):
            return error.localizedDescription

        case .success(let method):
            switch method {
            case .existingAccount:
                return NSLocalizedString(
                    "SUBHEAD_TITLE_SUCCESS",
                    tableName: "Login",
                    value: "Correct account number",
                    comment: ""
                )
            case .newAccount:
                return NSLocalizedString(
                    "SUBHEAD_TITLE_CREATED_ACCOUNT",
                    tableName: "Login",
                    value: "Account created",
                    comment: ""
                )
            }
        }
    }
}

// MARK: - AccountInputGroupViewDelegate

extension LoginViewController: AccountInputGroupViewDelegate {
    func accountInputGroupViewShouldRemoveLastUsedAccount(_ view: AccountInputGroupView) -> Bool {
        do {
            try SettingsManager.setLastUsedAccount(nil)
            return true
        } catch {
            self.logger.error(chainedError: AnyChainedError(error),
                              message: "Failed to remove last used account.")
            return false
        }
    }

    func accountInputGroupViewShouldAttemptLogin(_ view: AccountInputGroupView) {
        attemptLogin()
    }
}
