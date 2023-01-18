//
//  LoginViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 19/03/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import MullvadTypes
import Operations
import UIKit

enum LoginAction {
    case useExistingAccount(String)
    case createAccount

    var setAccountAction: SetAccountAction {
        switch self {
        case let .useExistingAccount(accountNumber):
            return .existing(accountNumber)
        case .createAccount:
            return .new
        }
    }
}

enum LoginState {
    case `default`
    case authenticating(LoginAction)
    case failure(Error)
    case success(LoginAction)
}

protocol LoginViewControllerDelegate: AnyObject {
    func loginViewController(
        _ controller: LoginViewController,
        shouldHandleLoginAction action: LoginAction,
        completion: @escaping (OperationCompletion<StoredAccountData?, Error>) -> Void
    )

    func loginViewControllerDidFinishLogin(_ controller: LoginViewController)
}

class LoginViewController: UIViewController, RootContainment {
    private lazy var contentView: LoginContentView = {
        let view = LoginContentView(frame: self.view.bounds)
        view.translatesAutoresizingMaskIntoConstraints = false
        return view
    }()

    private lazy var accountInputAccessoryCancelButton = UIBarButtonItem(
        barButtonSystemItem: .cancel,
        target: self,
        action: #selector(cancelLogin)
    )

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
            self.accountInputAccessoryLoginButton,
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
            contentView.accountInputGroup.textField.inputAccessoryView = self
                .accountInputAccessoryToolbar
        } else {
            contentView.accountInputGroup.textField.inputAccessoryView = nil
        }

        updateDisplayedMessage()
        updateStatusIcon()
        updateKeyboardToolbar()

        let notificationCenter = NotificationCenter.default

        contentView.createAccountButton.addTarget(
            self,
            action: #selector(createNewAccount),
            for: .touchUpInside
        )

        notificationCenter.addObserver(
            self,
            selector: #selector(textDidChange(_:)),
            name: UITextField.textDidChangeNotification,
            object: contentView.accountInputGroup.textField
        )
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

    func start(action: LoginAction) {
        beginLogin(action)

        delegate?
            .loginViewController(self, shouldHandleLoginAction: action) { [weak self] completion in
                switch completion {
                case let .success(accountData):
                    if case .createAccount = action {
                        self?.contentView.accountInputGroup.setAccount(accountData?.number ?? "")
                    }

                    self?.endLogin(.success(action))
                case let .failure(error):
                    self?.endLogin(.failure(error))
                case .cancelled:
                    self?.endLogin(.default)
                }
            }
    }

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

    @objc private func cancelLogin() {
        view.endEditing(true)
    }

    @objc private func doLogin() {
        let accountNumber = contentView.accountInputGroup.parsedToken

        start(action: .useExistingAccount(accountNumber))
    }

    @objc private func createNewAccount() {
        start(action: .createAccount)
    }

    // MARK: - Private

    private func updateLastUsedAccount() {
        do {
            let accountNumber = try SettingsManager.getLastUsedAccount()

            contentView.accountInputGroup.setLastUsedAccount(accountNumber, animated: false)
        } catch {
            logger.error(
                error: error,
                message: "Failed to update last used account."
            )
        }
    }

    private func loginStateDidChange() {
        contentView.accountInputGroup.setLoginState(loginState, animated: true)

        updateDisplayedMessage()
        updateStatusIcon()
        updateCreateButtonEnabled()
    }

    private func updateStatusIcon() {
        switch loginState {
        case .failure:
            contentView.statusActivityView.state = .failure
        case .success:
            contentView.statusActivityView.state = .success
        case .authenticating:
            contentView.statusActivityView.state = .activity
        case .default:
            contentView.statusActivityView.state = .hidden
        }
    }

    private func beginLogin(_ action: LoginAction) {
        loginState = .authenticating(action)

        view.endEditing(true)
    }

    private func endLogin(_ nextLoginState: LoginState) {
        let oldLoginState = loginState

        loginState = nextLoginState

        if case .authenticating(.useExistingAccount) = oldLoginState, case .failure = loginState {
            contentView.accountInputGroup.textField.becomeFirstResponder()
        } else if case .success = loginState {
            // Navigate to the main view after 1s delay
            DispatchQueue.main.asyncAfter(deadline: .now() + .seconds(1)) {
                self.delegate?.loginViewControllerDidFinishLogin(self)
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
            if case .pad = traitCollection.userInterfaceIdiom {
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

        case let .authenticating(method):
            switch method {
            case .useExistingAccount:
                return NSLocalizedString(
                    "SUBHEAD_TITLE_AUTHENTICATING",
                    tableName: "Login",
                    value: "Checking account number",
                    comment: ""
                )
            case .createAccount:
                return NSLocalizedString(
                    "SUBHEAD_TITLE_CREATING_ACCOUNT",
                    tableName: "Login",
                    value: "Creating new account",
                    comment: ""
                )
            }

        case let .failure(error):
            return (error as? DisplayError)?.displayErrorDescription ?? error.localizedDescription

        case let .success(method):
            switch method {
            case .useExistingAccount:
                return NSLocalizedString(
                    "SUBHEAD_TITLE_SUCCESS",
                    tableName: "Login",
                    value: "Correct account number",
                    comment: ""
                )
            case .createAccount:
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
            logger.error(
                error: error,
                message: "Failed to remove last used account."
            )
            return false
        }
    }

    func accountInputGroupViewShouldAttemptLogin(_ view: AccountInputGroupView) {
        attemptLogin()
    }
}
