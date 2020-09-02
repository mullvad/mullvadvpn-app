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

class LoginViewController: UIViewController, RootContainment {

    @IBOutlet var keyboardToolbar: UIToolbar!
    @IBOutlet var keyboardToolbarLoginButton: UIBarButtonItem!
    @IBOutlet var accountInputGroup: AccountInputGroupView!
    @IBOutlet var accountTextField: AccountTextField!
    @IBOutlet var titleLabel: UILabel!
    @IBOutlet var messageLabel: UILabel!
    @IBOutlet var loginForm: UIView!
    @IBOutlet var loginFormWrapperBottomConstraint: NSLayoutConstraint!
    @IBOutlet var activityIndicator: SpinnerActivityIndicatorView!
    @IBOutlet var statusImageView: UIImageView!
    @IBOutlet var createAccountButton: AppButton!

    private let logger = Logger(label: "LoginViewController")

    private var loginState = LoginState.default {
        didSet {
            loginStateDidChange()
        }
    }

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

        accountTextField.inputAccessoryView = keyboardToolbar
        accountTextField.attributedPlaceholder = NSAttributedString(
            string: "0000 0000 0000 0000",
            attributes: [.foregroundColor: UIColor.lightGray])

        updateDisplayedMessage()
        updateStatusIcon()
        updateKeyboardToolbar()

        let notificationCenter = NotificationCenter.default

        notificationCenter.addObserver(self,
                                       selector: #selector(keyboardWillShow(_:)),
                                       name: UIWindow.keyboardWillShowNotification,
                                       object: nil)
        notificationCenter.addObserver(self,
                                       selector: #selector(keyboardWillChangeFrame(_:)),
                                       name: UIWindow.keyboardWillChangeFrameNotification,
                                       object: nil)
        notificationCenter.addObserver(self,
                                       selector: #selector(keyboardWillHide(_:)),
                                       name: UIWindow.keyboardWillHideNotification,
                                       object: nil)

        notificationCenter.addObserver(self,
                                       selector: #selector(textDidBeginEditing(_:)),
                                       name: UITextField.textDidBeginEditingNotification,
                                       object: accountTextField)

        notificationCenter.addObserver(self,
                                       selector: #selector(textDidEndEditing(_:)),
                                       name: UITextField.textDidEndEditingNotification,
                                       object: accountTextField)

        notificationCenter.addObserver(self,
                                       selector: #selector(textDidChange(_:)),
                                       name: UITextField.textDidChangeNotification,
                                       object: accountTextField)
    }

    // MARK: - Public

    func reset() {
        loginState = .default
        accountTextField.autoformattingText = ""
        updateKeyboardToolbar()
    }

    // MARK: - Keyboard notifications

    @objc private func keyboardWillShow(_ notification: Notification) {
        guard let keyboardFrameValue = notification.userInfo?[UIWindow.keyboardFrameEndUserInfoKey] as? NSValue else { return }

        makeLoginFormVisible(keyboardFrame: keyboardFrameValue.cgRectValue)
    }

    @objc private func keyboardWillChangeFrame(_ notification: Notification) {
        guard let keyboardFrameValue = notification.userInfo?[UIWindow.keyboardFrameEndUserInfoKey] as? NSValue else { return }

        makeLoginFormVisible(keyboardFrame: keyboardFrameValue.cgRectValue)
    }

    @objc private func keyboardWillHide(_ notification: Notification) {
        loginFormWrapperBottomConstraint.constant = 0
        view.layoutIfNeeded()
    }

    // MARK: - UITextField notifications

    @objc func textDidBeginEditing(_ notification: Notification) {
        updateStatusIcon()
    }

    @objc func textDidEndEditing(_ notification: Notification) {
        updateStatusIcon()
    }

    @objc func textDidChange(_ notification: Notification) {
        // Reset the text style as user start typing
        if case .failure = loginState {
            loginState = .default
        }

        // Enable the log in button in the keyboard toolbar
        updateKeyboardToolbar()
    }

    // MARK: - Actions

    @IBAction func cancelLogin() {
        view.endEditing(true)
    }

    @IBAction func doLogin() {
        let accountToken = accountTextField.parsedToken

        beginLogin(method: .existingAccount)

        Account.shared.login(with: accountToken) { (result) in
            switch result {
            case .success:
                self.endLogin(.success(.existingAccount))

            case .failure(let error):
                self.logger.error(chainedError: error, message: "Failed to log in with existing account")

                self.endLogin(.failure(error))
            }
        }
    }

    @IBAction func createNewAccount() {
        beginLogin(method: .newAccount)

        accountTextField.autoformattingText = ""
        updateKeyboardToolbar()

        Account.shared.loginWithNewAccount { (result) in
            switch result {
            case .success(let response):
                self.accountTextField.autoformattingText = response.token

                self.endLogin(.success(.newAccount))
            case .failure(let error):
                self.logger.error(chainedError: error, message: "Failed to log in with new account")

                self.endLogin(.failure(error))
            }
        }
    }

    // MARK: - Private

    private func loginStateDidChange() {
        accountInputGroup.loginState = loginState

        // Keep the settings button disabled to prevent user from going to settings while
        // authentication or during the delay after the successful login and transition to the main
        // controller.
        switch loginState {
        case .authenticating:
            activityIndicator.startAnimating()
            createAccountButton.isEnabled = false

            // Fallthrough to make sure that the settings button is disabled
            // in .authenticating and .success cases.
            fallthrough

        case .success:
            rootContainerController?.setEnableSettingsButton(false)

        case .default, .failure:
            rootContainerController?.setEnableSettingsButton(true)
            createAccountButton.isEnabled = true
            activityIndicator.stopAnimating()
        }

        updateDisplayedMessage()
        updateStatusIcon()
    }

    private func updateStatusIcon() {
        switch loginState {
        case .failure:
            let opacity: CGFloat = self.accountTextField.isEditing ? 0 : 1
            statusImageView.image = UIImage(named: "IconFail")
            animateStatusImage(to: opacity)

        case .success:
            statusImageView.image = UIImage(named: "IconSuccess")
            animateStatusImage(to: 1)

        case .default, .authenticating:
            animateStatusImage(to: 0)
        }
    }

    private func animateStatusImage(to alpha: CGFloat) {
        UIView.animate(withDuration: 0.25) {
            self.statusImageView.alpha = alpha
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
            accountTextField.becomeFirstResponder()
        } else if case .success = loginState {
            // Navigate to the main view after 1s delay
            DispatchQueue.main.asyncAfter(deadline: .now() + .seconds(1)) {
                self.rootContainerController?.setEnableSettingsButton(true)

                self.performSegue(withIdentifier: SegueIdentifier.Login.showConnect.rawValue,
                                  sender: self)
            }
        }
    }

    private func updateDisplayedMessage() {
        titleLabel.text = loginState.localizedTitle
        messageLabel.text = loginState.localizedMessage
    }

    private func updateKeyboardToolbar() {
        let accountTokenLength = accountTextField.parsedToken.count
        let enableButton = accountTokenLength >= kMinimumAccountTokenLength

        keyboardToolbarLoginButton.isEnabled = enableButton
    }

    private func makeLoginFormVisible(keyboardFrame: CGRect) {
        let convertedKeyboardFrame = view.convert(keyboardFrame, from: nil)
        let (_, remainder) = view.frame.divided(atDistance: convertedKeyboardFrame.minY, from: CGRectEdge.minYEdge)

        loginFormWrapperBottomConstraint.constant = remainder.height
        view.layoutIfNeeded()
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
            case .tunnelConfiguration:
                return NSLocalizedString("Internal error", comment: "")
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
