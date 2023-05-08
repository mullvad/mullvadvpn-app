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

enum LoginState {
    case `default`
    case authenticating(LoginAction)
    case failure(LoginAction, Error)
    case success(LoginAction)
}

enum LoginAction {
    case useExistingAccount(String)
    case createAccount
}

enum EndLoginAction {
    /// Do nothing.
    case nothing

    /// Set focus on account text field.
    case activateTextField

    /// Wait for promise before showing login error.
    case wait(Promise<Void, Error>)
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

    private let interactor: LoginInteractor

    var didFinishLogin: ((LoginAction, Error?) -> EndLoginAction)?

    override var preferredStatusBarStyle: UIStatusBarStyle {
        return .lightContent
    }

    var preferredHeaderBarPresentation: HeaderBarPresentation {
        return HeaderBarPresentation(style: .transparent, showsDivider: false)
    }

    var prefersHeaderBarHidden: Bool {
        return false
    }

    var prefersNotificationBarHidden: Bool {
        return true
    }

    init(interactor: LoginInteractor) {
        self.interactor = interactor
        super.init(nibName: nil, bundle: nil)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
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

        contentView.accountInputGroup.didRemoveLastUsedAccount = { [weak self] in
            self?.interactor.removeLastUsedAccount()
        }

        contentView.accountInputGroup.didEnterAccount = { [weak self] in
            self?.attemptLogin()
        }

        contentView.accountInputGroup.setOnReturnKey { [weak self] _ in
            guard let self = self else { return true }

            return self.attemptLogin()
        }

        // There is no need to set the input accessory toolbar on iPad since it has a dedicated
        // button to dismiss the keyboard.
        if case .phone = UIDevice.current.userInterfaceIdiom {
            contentView.accountInputGroup.textField.inputAccessoryView = accountInputAccessoryToolbar
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

        switch action {
        case .createAccount:
            interactor.createAccount { [weak self] result in
                if let newAccountNumber = result.value {
                    self?.contentView.accountInputGroup.setAccount(newAccountNumber)
                }

                self?.endLogin(action: action, error: result.error)
            }

        case let .useExistingAccount(accountNumber):
            interactor.setAccount(accountNumber: accountNumber) { [weak self] error in
                self?.endLogin(action: action, error: error)
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
        contentView.accountInputGroup.setLastUsedAccount(
            interactor.getLastUsedAccount(),
            animated: false
        )
    }

    private func loginStateDidChange() {
        contentView.accountInputGroup.setLoginState(loginState, animated: true)

        updateDisplayedMessage()
        updateStatusIcon()
        updateCreateButtonEnabled()
    }

    private func updateStatusIcon() {
        contentView.statusActivityView.state = loginState.statusActivityState
    }

    private func beginLogin(_ action: LoginAction) {
        loginState = .authenticating(action)

        view.endEditing(true)
    }

    private func endLogin(action: LoginAction, error: Error?) {
        let nextLoginState: LoginState = error.map { .failure(action, $0) } ?? .success(action)

        let endAction = didFinishLogin?(action, error) ?? .nothing

        switch endAction {
        case .activateTextField:
            contentView.accountInputGroup.textField.becomeFirstResponder()
            loginState = nextLoginState

        case .nothing:
            loginState = nextLoginState

        case let .wait(promise):
            promise.observe { result in
                self.loginState = result.error.map { .failure(action, $0) } ?? nextLoginState
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
            // since it's likely overlaid by keyboard.
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

        case let .failure(_, error):
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

    var statusActivityState: StatusActivityView.State {
        switch self {
        case .failure:
            return .failure
        case .success:
            return .success
        case .authenticating:
            return .activity
        case .default:
            return .hidden
        }
    }
}
