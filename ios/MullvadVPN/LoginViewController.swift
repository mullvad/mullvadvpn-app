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

class LoginViewController: UIViewController, HeaderBarViewControllerDelegate {

    @IBOutlet var keyboardToolbar: UIToolbar!
    @IBOutlet var accountTextField: UITextField!
    @IBOutlet var loginForm: UIView!
    @IBOutlet var loginFormWrapperBottomConstraint: NSLayoutConstraint!
    @IBOutlet var activityIndicator: SpinnerActivityIndicatorView!

    private let procedureQueue = ProcedureQueue()

    override var preferredStatusBarStyle: UIStatusBarStyle {
        return .lightContent
    }

    override func prepare(for segue: UIStoryboardSegue, sender: Any?) {
        if case .embedHeader? = SegueIdentifier.Login.from(segue: segue) {
            let headerBarController = segue.destination as? HeaderBarViewController
            headerBarController?.delegate = self
        }
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        accountTextField.inputAccessoryView = keyboardToolbar

        NotificationCenter.default.addObserver(self, selector: #selector(keyboardWillShow(_:)), name: UIWindow.keyboardWillShowNotification, object: nil)
        NotificationCenter.default.addObserver(self, selector: #selector(keyboardWillChangeFrame(_:)), name: UIWindow.keyboardWillChangeFrameNotification, object: nil)
        NotificationCenter.default.addObserver(self, selector: #selector(keyboardWillHide(_:)), name: UIWindow.keyboardWillHideNotification, object: nil)
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

    // MARK: - IBActions

    @IBAction func cancelLogin() {
        view.endEditing(true)
    }

    @IBAction func doLogin() {
        let accountToken = accountTextField.text ?? ""

        beginLoginAnimations()

        verifyAccount(accountToken: accountToken) { [weak self] (result) in
            guard let self = self else { return }

            switch result {
            case .success(let verification):
                self.didReceiveAccountVerification(verification)
            case .failure(let error):
                os_log(.error, "Failed to request the account verification: %{public}s", error.localizedDescription)
            }

            self.endLoginAnimations()
        }

    }

    // MARK: - Private

    private func verifyAccount(accountToken: String, completion: @escaping (Result<AccountVerification, Error>) -> Void) {
        let delayProcedure = DelayProcedure(by: 1)
        let verifyProcedure = MullvadAPI.verifyAccountToken(accountToken)

        verifyProcedure.addDependency(delayProcedure)
        verifyProcedure.addDidFinishBlockObserver(synchronizedWith: DispatchQueue.main) { (procedure, error) in
            let result = error.flatMap({ .failure($0) })
                ?? Result(catching: { try procedure.output.success.unwrap() })

            completion(result)
        }

        procedureQueue.addOperations([delayProcedure, verifyProcedure])
    }

    private func beginLoginAnimations() {
        activityIndicator.isAnimating = true
        accountTextField.isEnabled = false

        view.endEditing(true)
    }

    private func endLoginAnimations() {
        activityIndicator.isAnimating = false
        accountTextField.isEnabled = true
    }

    private func didReceiveAccountVerification(_ result: AccountVerification) {
        switch result {
        case .deferred(let networkError):
            print("Network error: \(networkError.localizedDescription)")

            performSegue(withIdentifier: "ShowConnect", sender: self)

        case .invalid:
            print("Invalid account token")

            // TODO: Show the error

        case .verified(let expiryDate):
            print("All good! The account expires on \(expiryDate)")

            performSegue(withIdentifier: "ShowConnect", sender: self)
        }
    }

    private func makeLoginFormVisible(keyboardFrame: CGRect) {
        let convertedKeyboardFrame = view.convert(keyboardFrame, from: nil)
        let (_, remainder) = view.frame.divided(atDistance: convertedKeyboardFrame.minY, from: CGRectEdge.minYEdge)

        loginFormWrapperBottomConstraint.constant = remainder.height
        view.layoutIfNeeded()
    }
}

