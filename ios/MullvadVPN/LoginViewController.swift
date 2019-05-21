//
//  LoginViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 19/03/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import UIKit
import ProcedureKit
import os.log

private let kMinimumAccountTokenLength = 10
private let kValidAccountTokenCharacterSet = CharacterSet(charactersIn: "01234567890")

class LoginViewController: UIViewController, HeaderBarViewControllerDelegate, UITextFieldDelegate {

    @IBOutlet var keyboardToolbar: UIToolbar!
    @IBOutlet var keyboardToolbarLoginButton: UIBarButtonItem!
    @IBOutlet var accountInputGroup: AccountInputGroupView!
    @IBOutlet var accountTextField: UITextField!
    @IBOutlet var messageLabel: UILabel!
    @IBOutlet var loginForm: UIView!
    @IBOutlet var loginFormWrapperBottomConstraint: NSLayoutConstraint!
    @IBOutlet var activityIndicator: SpinnerActivityIndicatorView!
    @IBOutlet var statusImageView: UIImageView!

    private weak var headerBarController: HeaderBarViewController?

    private let procedureQueue = ProcedureQueue()
    private var loginState = LoginState.default {
        didSet {
            loginStateDidChange()
        }
    }

    override var preferredStatusBarStyle: UIStatusBarStyle {
        return .lightContent
    }

    override func prepare(for segue: UIStoryboardSegue, sender: Any?) {
        if case .embedHeader? = SegueIdentifier.Login.from(segue: segue) {
            headerBarController = segue.destination as? HeaderBarViewController
            headerBarController?.delegate = self
        }
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        accountTextField.inputAccessoryView = keyboardToolbar

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

    // MARK: - HeaderBarViewControllerDelegate

    func headerBarViewControllerShouldOpenSettings(_ controller: HeaderBarViewController) {
        performSegue(withIdentifier: SegueIdentifier.Login.showSettings.rawValue, sender: self)
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

    // MARK: - UITextFieldDelegate

    func textField(_ textField: UITextField, shouldChangeCharactersIn range: NSRange, replacementString string: String) -> Bool {
        // prevent the change if the replacement string contains disallowed characters
        return string.unicodeScalars.allSatisfy { kValidAccountTokenCharacterSet.contains($0) }
    }

    // MARK: - IBActions

    @IBAction func cancelLogin() {
        view.endEditing(true)
    }

    @IBAction func doLogin() {
        let accountToken = accountTextField.text ?? ""

        beginLogin()

        verifyAccount(accountToken: accountToken) { [weak self] (result) in
            guard let self = self else { return }

            switch result {
            case .success:
                self.endLogin(.success)

            case .failure(let error):
                self.endLogin(.failure(error))
            }
        }
    }

    @IBAction func openCreateAccount() {
        UIApplication.shared.open(WebLinks.createAccountURL, options: [:])
    }

    // MARK: - Private

    private func verifyAccount(accountToken: String, completion: @escaping (Result<(), Error>) -> Void) {
        let delayProcedure = DelayProcedure(by: 1)
        let loginProcedure = Account.login(with: accountToken)

        loginProcedure.addDependency(delayProcedure)
        loginProcedure.addDidFinishBlockObserver(synchronizedWith: DispatchQueue.main) { (_, error) in
            completion(error.flatMap({ .failure($0) }) ?? .success(()))
        }

        procedureQueue.addOperations([delayProcedure, loginProcedure])
    }

    private func loginStateDidChange() {
        accountInputGroup.loginState = loginState

        if case .authenticating = loginState {
            activityIndicator.isAnimating = true
            headerBarController?.settingsButton.isEnabled = false
        } else {
            activityIndicator.isAnimating = false
            headerBarController?.settingsButton.isEnabled = true
        }

        updateDisplayedMessage()
        updateStatusIcon()
    }

    private func updateStatusIcon() {
        switch loginState {
        case .failure:
            let opacity: CGFloat = self.accountTextField.isEditing ? 0 : 1
            statusImageView.image = UIImage(imageLiteralResourceName: "IconFail")
            animateStatusImage(to: opacity)

        case .success:
            statusImageView.image = UIImage(imageLiteralResourceName: "IconSuccess")
            animateStatusImage(to: 1)

        default:
            animateStatusImage(to: 0)
        }
    }

    private func animateStatusImage(to alpha: CGFloat) {
        UIView.animate(withDuration: 0.25) {
            self.statusImageView.alpha = alpha
        }
    }

    private func beginLogin() {
        loginState = .authenticating

        view.endEditing(true)
    }

    private func endLogin(_ nextLoginState: LoginState) {
        loginState = nextLoginState

        if case .failure = loginState {
            accountTextField.becomeFirstResponder()
        } else if case .success = loginState {
            // Navigate to the main view after 1s delay
            DispatchQueue.main.asyncAfter(deadline: .now() + .seconds(1)) {
                self.performSegue(withIdentifier: SegueIdentifier.Login.showConnect.rawValue,
                                  sender: self)
            }
        }
    }

    private func updateDisplayedMessage() {
        messageLabel.text = loginState.localizedDescription
    }

    private func updateKeyboardToolbar() {
        let accountTokenLength = accountTextField.text?.count ?? 0
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
    var localizedDescription: String {
        switch self {
        case .default:
            return NSLocalizedString("Enter your account number", tableName: "Login", comment: "")

        case .authenticating:
            return NSLocalizedString("Checking account number", tableName: "Login", comment: "")

        case .failure(let error):
            if case .invalidAccount? = error as? Account.Error {
                return NSLocalizedString("Invalid account number", tableName: "Login", comment: "")
            } else {
                return NSLocalizedString("Internal error", tableName: "Login", comment: "")
            }

        case .success:
            return NSLocalizedString("Correct account number", tableName: "Login", comment: "")
        }
    }
}
