//
//  ProblemReportViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 15/09/2020.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import MullvadTypes
import Operations
import UIKit

final class ProblemReportViewController: UIViewController, UITextFieldDelegate {
    private let alertPresenter: AlertPresenter
    let interactor: ProblemReportInteractor

    var textViewKeyboardResponder: AutomaticKeyboardResponder?
    var scrollViewKeyboardResponder: AutomaticKeyboardResponder?
    var showsSubmissionOverlay = false
    var includeAccountTokenInLogs = false

    /// Constraints used when description text view is active
    var activeMessageTextViewConstraints = [NSLayoutConstraint]()
    /// Constraints used when description text view is inactive
    var inactiveMessageTextViewConstraints = [NSLayoutConstraint]()
    /// Flag indicating when the text view is expanded to fill the entire view
    var isMessageTextViewExpanded = false

    static var persistentViewModel = ProblemReportViewModel()

    /// Scroll view
    lazy var scrollView: UIScrollView = { makeScrollView() }()
    /// Scroll view content container
    lazy var containerView: UIView = { makeContainerView() }()
    /// Subheading label displayed below navigation bar
    lazy var subheaderLabel: UILabel = { makeSubheaderLabel() }()
    lazy var includeDeviceTokenCheckbox: UIStackView = { makeCheckboxStackView() }()
    lazy var emailTextField: CustomTextField = { makeEmailTextField() }()
    lazy var messageTextView: CustomTextView = { makeMessageTextView() }()
    /// Container view for text input fields
    lazy var textFieldsHolder: UIView = { makeTextFieldsHolder() }()
    /// Placeholder view used to fill the space within the scroll view when the text view is
    /// expanded to fill the entire view
    lazy var messagePlaceholder: UIView = { makeMessagePlaceholderView() }()
    /// Footer stack view that contains action buttons
    lazy var buttonsStackView: UIStackView = { makeButtonsStackView() }()
    lazy var viewLogsButton: AppButton = { makeViewLogsButton() }()
    lazy var sendButton: AppButton = { makeSendButton() }()
    lazy var emailAccessoryToolbar: UIToolbar = makeKeyboardToolbar(
        canGoBackward: false,
        canGoForward: true
    )
    lazy var messageAccessoryToolbar: UIToolbar = makeKeyboardToolbar(
        canGoBackward: true,
        canGoForward: false
    )

    lazy var submissionOverlayView: ProblemReportSubmissionOverlayView = { makeSubmissionOverlayView() }()

    var checkboxView: CheckboxView!
    var reduceAnonymityWarningView: ReduceAnonymityWarningView!

    // MARK: - View lifecycle

    override var preferredStatusBarStyle: UIStatusBarStyle { .lightContent }
    // Allow dismissing the keyboard in .formSheet presentation style
    override var disablesAutomaticKeyboardDismissal: Bool { false }

    init(interactor: ProblemReportInteractor, alertPresenter: AlertPresenter) {
        self.interactor = interactor
        self.alertPresenter = alertPresenter

        super.init(nibName: nil, bundle: nil)
    }

    required init?(coder: NSCoder) { fatalError("init(coder:) has not been implemented") }

    override func viewDidLoad() {
        super.viewDidLoad()

        view.backgroundColor = .secondaryColor
        view.setAccessibilityIdentifier(.problemReportView)

        navigationItem.title = ProblemReportViewModel.navigationTitle

        textViewKeyboardResponder = AutomaticKeyboardResponder(targetView: messageTextView)
        scrollViewKeyboardResponder = AutomaticKeyboardResponder(targetView: scrollView)

        // Make sure that the user can't easily dismiss the controller on iOS 13 and above
        isModalInPresentation = true

        // Set hugging & compression priorities so that description text view wants to grow
        emailTextField.setContentHuggingPriority(.defaultHigh, for: .vertical)
        emailTextField.setContentCompressionResistancePriority(.defaultHigh, for: .vertical)
        messageTextView.setContentHuggingPriority(.defaultLow, for: .vertical)
        messageTextView.setContentCompressionResistancePriority(.defaultLow, for: .vertical)

        emailTextField.setAccessibilityIdentifier(.problemReportEmailTextField)
        messageTextView.setAccessibilityIdentifier(.problemReportMessageTextView)

        addConstraints()
        registerForNotifications()
        loadPersistentViewModel()
    }

    // MARK: - Actions

    @objc func focusEmailTextField() {
        emailTextField.becomeFirstResponder()
    }

    @objc func focusDescriptionTextView() {
        messageTextView.becomeFirstResponder()
    }

    @objc func dismissKeyboard() {
        view.endEditing(false)
    }

    @objc func handleSendButtonTap() {
        let proceedWithSubmission = {
            self.sendProblemReport()
        }

        if Self.persistentViewModel.email.isEmpty {
            presentEmptyEmailConfirmationAlert { shouldSend in
                if shouldSend {
                    proceedWithSubmission()
                }
            }
        } else {
            proceedWithSubmission()
        }
    }

    @objc func handleViewLogsButtonTap() {
        let reviewController = ProblemReportReviewViewController(interactor: interactor)
        let navigationController = CustomNavigationController(rootViewController: reviewController)

        present(navigationController, animated: true)
    }

    // MARK: - Private

    private func registerForNotifications() {
        let notificationCenter = NotificationCenter.default
        notificationCenter.addObserver(
            self,
            selector: #selector(emailTextFieldDidChange),
            name: UITextField.textDidChangeNotification,
            object: emailTextField
        )
        notificationCenter.addObserver(
            self,
            selector: #selector(messageTextViewDidBeginEditing),
            name: UITextView.textDidBeginEditingNotification,
            object: messageTextView
        )
        notificationCenter.addObserver(
            self,
            selector: #selector(messageTextViewDidEndEditing),
            name: UITextView.textDidEndEditingNotification,
            object: messageTextView
        )
        notificationCenter.addObserver(
            self,
            selector: #selector(messageTextViewDidChange),
            name: UITextView.textDidChangeNotification,
            object: messageTextView
        )
    }

    private func presentEmptyEmailConfirmationAlert(completion: @escaping (Bool) -> Void) {
        let presentation = AlertPresentation(
            id: "problem-report-alert",
            icon: .alert,
            message: ProblemReportViewModel.emptyEmailAlertWarning,
            buttons: [
                AlertAction(
                    title: ProblemReportViewModel.confirmEmptyEmailTitle,
                    style: .destructive,
                    handler: {
                        completion(true)
                    }
                ),
                AlertAction(
                    title: ProblemReportViewModel.cancelEmptyEmailTitle,
                    style: .default,
                    handler: {
                        completion(false)
                    }
                ),
            ]
        )

        alertPresenter.showAlert(presentation: presentation, animated: true)
    }

    // MARK: - Data model

    private func loadPersistentViewModel() {
        emailTextField.text = Self.persistentViewModel.email
        messageTextView.text = Self.persistentViewModel.message

        validateForm()
    }

    private func updatePersistentViewModel() {
        Self.persistentViewModel = ProblemReportViewModel(
            email: emailTextField.text ?? "",
            message: messageTextView.text,
            includeAccountTokenInLogs: includeAccountTokenInLogs
        )

        validateForm()
    }

    private func setPopGestureEnabled(_ isEnabled: Bool) {
        navigationController?.interactivePopGestureRecognizer?.isEnabled = isEnabled
    }

    private func clearPersistentViewModel() {
        Self.persistentViewModel = ProblemReportViewModel()
    }

    // MARK: - Form validation

    private func validateForm() {
        sendButton.isEnabled = Self.persistentViewModel.isValid
    }

    // MARK: - Problem submission progress handling

    private func willSendProblemReport() {
        showSubmissionOverlay()

        submissionOverlayView.state = .sending
        navigationItem.setHidesBackButton(true, animated: true)
    }

    private func didSendProblemReport(
        viewModel: ProblemReportViewModel,
        completion: Result<Void, Error>
    ) {
        switch completion {
        case .success:
            submissionOverlayView.state = .sent(viewModel.email)

            // Clear persistent view model upon successful submission
            clearPersistentViewModel()

        case let .failure(error):
            if let error = error as? OperationError, error == .cancelled {
                hideSubmissionOverlay()
            } else {
                submissionOverlayView.state = .failure(error)
            }
        }

        navigationItem.setHidesBackButton(false, animated: true)
    }

    // MARK: - Problem report submission helpers

    func sendProblemReport() {
        let viewModel = Self.persistentViewModel

        willSendProblemReport()

        interactor.sendReport(
            email: viewModel.email,
            message: viewModel.message,
            includeAccountTokenInLogs: includeAccountTokenInLogs
        ) { [weak self] completion in
            Task { @MainActor in
                self?.didSendProblemReport(viewModel: viewModel, completion: completion)
            }
        }
    }

    // MARK: - Input fields notifications

    func didToggleIncludeAccountTokenInLogs(_ includeTokenInLogs: Bool) {
        includeAccountTokenInLogs = includeTokenInLogs
    }

    @objc private func messageTextViewDidBeginEditing() {
        setDescriptionFieldExpanded(true)
        setPopGestureEnabled(false)
    }

    @objc private func messageTextViewDidEndEditing() {
        setDescriptionFieldExpanded(false)
        setPopGestureEnabled(true)
    }

    @objc private func messageTextViewDidChange() {
        updatePersistentViewModel()
    }

    @objc private func emailTextFieldDidChange() {
        updatePersistentViewModel()
    }

    // MARK: - UITextFieldDelegate

    func textFieldDidBeginEditing(_ textField: UITextField) {
        setPopGestureEnabled(false)
    }

    func textFieldDidEndEditing(_ textField: UITextField) {
        setPopGestureEnabled(true)
    }

    func textFieldShouldReturn(_ textField: UITextField) -> Bool {
        messageTextView.becomeFirstResponder()
        return false
    }
}
