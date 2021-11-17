//
//  ProblemReportViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 15/09/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import UIKit

class ProblemReportViewController: UIViewController, UITextFieldDelegate, ConditionalNavigation {

    private var textViewKeyboardResponder: AutomaticKeyboardResponder?
    private var scrollViewKeyboardResponder: AutomaticKeyboardResponder?

    private lazy var consolidatedLog: ConsolidatedApplicationLog = {
        let securityGroupIdentifier = ApplicationConfiguration.securityGroupIdentifier

        // TODO: make sure we redact old tokens
        let redactStrings = Account.shared.token.flatMap { [$0] } ?? []

        let report = ConsolidatedApplicationLog(
            redactCustomStrings: redactStrings,
            redactContainerPathsForSecurityGroupIdentifiers: [securityGroupIdentifier]
        )

        report.addLogFiles(fileURLs: ApplicationConfiguration.logFileURLs, includeLogBackups: true)

        return report
    }()

    /// Scroll view
    private lazy var scrollView: UIScrollView = {
        let scrollView = UIScrollView()
        scrollView.translatesAutoresizingMaskIntoConstraints = false
        scrollView.backgroundColor = .clear
        return scrollView
    }()

    /// Scroll view content container
    private lazy var containerView: UIView = {
        let containerView = UIView()
        containerView.translatesAutoresizingMaskIntoConstraints = false
        containerView.layoutMargins = UIMetrics.contentLayoutMargins
        containerView.backgroundColor = .clear
        return containerView
    }()

    /// Subheading label displayed below navigation bar
    private lazy var subheaderLabel: UILabel = {
        let textLabel = UILabel()
        textLabel.translatesAutoresizingMaskIntoConstraints = false
        textLabel.numberOfLines = 0
        textLabel.textColor = .white
        textLabel.text = NSLocalizedString(
            "SUBHEAD_LABEL",
            tableName: "ProblemReport",
            comment: "Subhead label displayed below navigation title."
        )
        return textLabel
    }()

    private lazy var emailTextField: CustomTextField = {
        let textField = CustomTextField()
        textField.translatesAutoresizingMaskIntoConstraints = false
        textField.delegate = self
        textField.keyboardType = .emailAddress
        textField.textContentType = .emailAddress
        textField.autocorrectionType = .no
        textField.autocapitalizationType = .none
        textField.smartInsertDeleteType = .no
        textField.returnKeyType = .next
        textField.borderStyle = .none
        textField.backgroundColor = .white
        textField.inputAccessoryView = emailAccessoryToolbar
        textField.font = UIFont.systemFont(ofSize: 17)
        textField.placeholder = NSLocalizedString(
            "EMAIL_TEXTFIELD_PLACEHOLDER",
            tableName: "ProblemReport",
            comment: "Placeholder for email text field."
        )

        return textField
    }()

    private lazy var messageTextView: CustomTextView = {
        let textView = CustomTextView()
        textView.translatesAutoresizingMaskIntoConstraints = false
        textView.backgroundColor = .white
        textView.inputAccessoryView = messageAccessoryToolbar
        textView.font = UIFont.systemFont(ofSize: 17)
        textView.placeholder = NSLocalizedString(
            "DESCRIPTION_TEXTVIEW_PLACEHOLDER",
            tableName: "ProblemReport",
            comment: "Placeholder for description text view."
        )
        textView.contentInsetAdjustmentBehavior = .never

        return textView
    }()

    /// Container view for text input fields
    private lazy var textFieldsHolder: UIView = {
        let view = UIView()
        view.translatesAutoresizingMaskIntoConstraints = false
        return view
    }()

    /// Constraints used when description text view is active
    private var activeMessageTextViewConstraints = [NSLayoutConstraint]()

    /// Constraints used when description text view is inactive
    private var inactiveMessageTextViewConstraints = [NSLayoutConstraint]()

    /// Flag indicating when the text view is expanded to fill the entire view
    private var isMessageTextViewExpanded = false

    /// Placeholder view used to fill the space within the scroll view when the text view is
    /// expanded to fill the entire view
    private lazy var messagePlaceholder: UIView = {
        let view = UIView()
        view.translatesAutoresizingMaskIntoConstraints = false
        view.backgroundColor = .clear
        return view
    }()

    /// Footer stack view that contains action buttons
    private lazy var buttonsStackView: UIStackView = {
        let stackView = UIStackView(arrangedSubviews: [self.viewLogsButton, self.sendButton])
        stackView.translatesAutoresizingMaskIntoConstraints = false
        stackView.axis = .vertical
        stackView.spacing = 18

        return stackView
    }()

    private lazy var viewLogsButton: AppButton = {
        let button = AppButton(style: .default)
        button.translatesAutoresizingMaskIntoConstraints = false
        button.setTitle(NSLocalizedString(
            "VIEW_APP_LOGS_BUTTON_TITLE",
            tableName: "ProblemReport",
            comment: "Title for button to view application logs"
        ), for: .normal)
        button.addTarget(self, action: #selector(handleViewLogsButtonTap), for: .touchUpInside)
        return button
    }()

    private lazy var sendButton: AppButton = {
        let button = AppButton(style: .success)
        button.translatesAutoresizingMaskIntoConstraints = false
        button.setTitle(NSLocalizedString(
            "SEND_BUTTON_TITLE",
            tableName: "ProblemReport",
            comment: "Title for button to send problem report."
        ), for: .normal)
        button.addTarget(self, action: #selector(handleSendButtonTap), for: .touchUpInside)
        return button
    }()

    private lazy var emailAccessoryToolbar: UIToolbar = {
        return makeKeyboardToolbar(canGoBackward: false, canGoForward: true)
    }()

    private lazy var messageAccessoryToolbar: UIToolbar = {
        return makeKeyboardToolbar(canGoBackward: true, canGoForward: false)
    }()

    private lazy var submissionOverlayView: ProblemReportSubmissionOverlayView = {
        let overlay = ProblemReportSubmissionOverlayView()
        overlay.translatesAutoresizingMaskIntoConstraints = false

        overlay.editButtonAction = { [weak self] in
            self?.hideSubmissionOverlay()
        }

        overlay.retryButtonAction = { [weak self] in
            self?.sendProblemReport()
        }

        return overlay
    }()

    // MARK: - View lifecycle

    override var preferredStatusBarStyle: UIStatusBarStyle {
        return .lightContent
    }

    override var disablesAutomaticKeyboardDismissal: Bool {
        // Allow dismissing the keyboard in .formSheet presentation style
        return false
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        view.backgroundColor = .secondaryColor

        navigationItem.title = NSLocalizedString("NAVIGATION_TITLE", tableName: "ProblemReport", comment: "Navigation title")

        textViewKeyboardResponder = AutomaticKeyboardResponder(targetView: messageTextView)
        scrollViewKeyboardResponder = AutomaticKeyboardResponder(targetView: scrollView)

        // Make sure that the user can't easily dismiss the controller on iOS 13 and above
        if #available(iOS 13.0, *) {
            isModalInPresentation = true
        }

        // Set hugging & compression priorities so that description text view wants to grow
        emailTextField.setContentHuggingPriority(.defaultHigh, for: .vertical)
        emailTextField.setContentCompressionResistancePriority(.defaultHigh, for: .vertical)
        messageTextView.setContentHuggingPriority(.defaultLow, for: .vertical)
        messageTextView.setContentCompressionResistancePriority(.defaultLow, for: .vertical)

        textFieldsHolder.addSubview(emailTextField)
        textFieldsHolder.addSubview(messagePlaceholder)
        textFieldsHolder.addSubview(messageTextView)

        view.addSubview(scrollView)
        scrollView.addSubview(containerView)
        containerView.addSubview(subheaderLabel)
        containerView.addSubview(textFieldsHolder)
        containerView.addSubview(buttonsStackView)

        addConstraints()
        registerForNotifications()

        loadPersistentViewModel()
    }

    override func viewSafeAreaInsetsDidChange() {
        super.viewSafeAreaInsetsDidChange()

        self.scrollViewKeyboardResponder?.updateContentInsets()
        self.textViewKeyboardResponder?.updateContentInsets()
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
            presentEmptyEmailConfirmationAlert { (shouldSend) in
                if shouldSend {
                    proceedWithSubmission()
                }
            }
        } else {
            proceedWithSubmission()
        }
    }

    @objc func handleViewLogsButtonTap() {
        let reviewController = ProblemReportReviewViewController(reportString: consolidatedLog.string)
        let navigationController = UINavigationController(rootViewController: reviewController)

        present(navigationController, animated: true)
    }

    // MARK: - Private

    private func registerForNotifications() {
        let notificationCenter = NotificationCenter.default
        notificationCenter.addObserver(self, selector: #selector(emailTextFieldDidChange),
                                       name: UITextField.textDidChangeNotification,
                                       object: emailTextField)
        notificationCenter.addObserver(self, selector: #selector(messageTextViewDidBeginEditing),
                                       name: UITextView.textDidBeginEditingNotification,
                                       object: messageTextView)
        notificationCenter.addObserver(self, selector: #selector(messageTextViewDidEndEditing),
                                       name: UITextView.textDidEndEditingNotification,
                                       object: messageTextView)
        notificationCenter.addObserver(self, selector: #selector(messageTextViewDidChange),
                                       name: UITextView.textDidChangeNotification,
                                       object: messageTextView)
    }

    private func makeKeyboardToolbar(canGoBackward: Bool, canGoForward: Bool) -> UIToolbar {
        var toolbarItems = UIBarButtonItem.makeKeyboardNavigationItems { (prevButton, nextButton) in
            prevButton.target = self
            prevButton.action = #selector(focusEmailTextField)
            prevButton.isEnabled = canGoBackward

            nextButton.target = self
            nextButton.action = #selector(focusDescriptionTextView)
            nextButton.isEnabled = canGoForward
        }

        toolbarItems.append(contentsOf: [
            UIBarButtonItem(barButtonSystemItem: .flexibleSpace, target: nil, action: nil),
            UIBarButtonItem(barButtonSystemItem: .done, target: self, action: #selector(dismissKeyboard))
        ])

        let toolbar = UIToolbar(frame: CGRect(x: 0, y: 0, width: 100, height: 44))
        toolbar.items = toolbarItems
        return toolbar
    }

    private func addConstraints() {
        self.activeMessageTextViewConstraints = [
            messageTextView.topAnchor.constraint(equalTo: view.topAnchor),
            messageTextView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            messageTextView.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            messageTextView.bottomAnchor.constraint(equalTo: view.bottomAnchor),
        ]

        self.inactiveMessageTextViewConstraints = [
            messageTextView.topAnchor.constraint(equalTo: emailTextField.bottomAnchor, constant: 12),
            messageTextView.leadingAnchor.constraint(equalTo: textFieldsHolder.leadingAnchor),
            messageTextView.trailingAnchor.constraint(equalTo: textFieldsHolder.trailingAnchor),
            messageTextView.bottomAnchor.constraint(equalTo: textFieldsHolder.bottomAnchor),
        ]

        var constraints = [
            subheaderLabel.topAnchor.constraint(equalTo: containerView.layoutMarginsGuide.topAnchor),
            subheaderLabel.leadingAnchor.constraint(equalTo: containerView.layoutMarginsGuide.leadingAnchor),
            subheaderLabel.trailingAnchor.constraint(equalTo: containerView.layoutMarginsGuide.trailingAnchor),

            textFieldsHolder.topAnchor.constraint(equalTo: subheaderLabel.bottomAnchor, constant: 24),
            textFieldsHolder.leadingAnchor.constraint(equalTo: containerView.layoutMarginsGuide.leadingAnchor),
            textFieldsHolder.trailingAnchor.constraint(equalTo: containerView.layoutMarginsGuide.trailingAnchor),

            buttonsStackView.topAnchor.constraint(equalTo: textFieldsHolder.bottomAnchor, constant: 18),
            buttonsStackView.leadingAnchor.constraint(equalTo: containerView.layoutMarginsGuide.leadingAnchor),
            buttonsStackView.trailingAnchor.constraint(equalTo: containerView.layoutMarginsGuide.trailingAnchor),
            buttonsStackView.bottomAnchor.constraint(equalTo: containerView.layoutMarginsGuide.bottomAnchor),

            emailTextField.topAnchor.constraint(equalTo: textFieldsHolder.topAnchor),
            emailTextField.leadingAnchor.constraint(equalTo: textFieldsHolder.leadingAnchor),
            emailTextField.trailingAnchor.constraint(equalTo: textFieldsHolder.trailingAnchor),

            messagePlaceholder.topAnchor.constraint(equalTo: emailTextField.bottomAnchor, constant: 12),
            messagePlaceholder.leadingAnchor.constraint(equalTo: textFieldsHolder.leadingAnchor),
            messagePlaceholder.trailingAnchor.constraint(equalTo: textFieldsHolder.trailingAnchor),
            messagePlaceholder.bottomAnchor.constraint(equalTo: textFieldsHolder.bottomAnchor),
            messagePlaceholder.heightAnchor.constraint(equalTo: messageTextView.heightAnchor),

            scrollView.frameLayoutGuide.topAnchor.constraint(equalTo: view.topAnchor),
            scrollView.frameLayoutGuide.bottomAnchor.constraint(equalTo: view.bottomAnchor),
            scrollView.frameLayoutGuide.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            scrollView.frameLayoutGuide.trailingAnchor.constraint(equalTo: view.trailingAnchor),

            scrollView.contentLayoutGuide.topAnchor.constraint(equalTo: containerView.topAnchor),
            scrollView.contentLayoutGuide.bottomAnchor.constraint(equalTo: containerView.bottomAnchor),
            scrollView.contentLayoutGuide.leadingAnchor.constraint(equalTo: containerView.leadingAnchor),
            scrollView.contentLayoutGuide.trailingAnchor.constraint(equalTo: containerView.trailingAnchor),

            scrollView.contentLayoutGuide.widthAnchor.constraint(equalTo: scrollView.frameLayoutGuide.widthAnchor),
            scrollView.contentLayoutGuide.heightAnchor.constraint(greaterThanOrEqualTo: scrollView.safeAreaLayoutGuide.heightAnchor),

            messageTextView.heightAnchor.constraint(greaterThanOrEqualToConstant: 150),
        ]

        constraints.append(contentsOf: self.inactiveMessageTextViewConstraints)

        NSLayoutConstraint.activate(constraints)
    }

    private func setDescriptionFieldExpanded(_ isExpanded: Bool) {
        // Make voice over ignore siblings when expanded
        self.messageTextView.accessibilityViewIsModal = isExpanded

        if isExpanded {
            // Disable the large title
            self.navigationItem.largeTitleDisplayMode = .never

            // Move the text view above scroll view
            view.addSubview(self.messageTextView)

            // Re-add old constraints
            NSLayoutConstraint.activate(self.inactiveMessageTextViewConstraints)

            // Do a layout pass
            view.layoutIfNeeded()

            // Swap constraints
            NSLayoutConstraint.deactivate(self.inactiveMessageTextViewConstraints)
            NSLayoutConstraint.activate(self.activeMessageTextViewConstraints)

            // Enable content inset adjustment on text view
            self.messageTextView.contentInsetAdjustmentBehavior = .always

            // Animate constraints & rounded corners on the text view
            animateDescriptionTextView(animations: {
                // Turn off rounded corners as the text view fills in the entire view
                self.messageTextView.roundCorners = false

                self.view.layoutIfNeeded()
            }) { (completed) in
                self.isMessageTextViewExpanded = true

                self.textViewKeyboardResponder?.updateContentInsets()

                // Tell accessibility engine to scan the new layout
                UIAccessibility.post(notification: .layoutChanged, argument: nil)
            }

        } else {
            // Re-enable the large title
            self.navigationItem.largeTitleDisplayMode = .automatic

            // Swap constraints
            NSLayoutConstraint.deactivate(self.activeMessageTextViewConstraints)
            NSLayoutConstraint.activate(self.inactiveMessageTextViewConstraints)

            // Animate constraints & rounded corners on the text view
            animateDescriptionTextView(animations: {
                // Turn on rounded corners as the text view returns back to where it was
                self.messageTextView.roundCorners = true

                self.view.layoutIfNeeded()
            }) { (completed) in
                // Revert the content adjustment behavior
                self.messageTextView.contentInsetAdjustmentBehavior = .never

                // Add the text view inside of the scroll view
                self.textFieldsHolder.addSubview(self.messageTextView)

                self.isMessageTextViewExpanded = false

                // Tell accessibility engine to scan the new layout
                UIAccessibility.post(notification: .layoutChanged, argument: nil)
            }
        }
    }

    private func animateDescriptionTextView(animations: @escaping () -> Void, completion: @escaping (Bool) -> Void) {
        UIView.animate(withDuration: 0.25, animations: animations) { (completed) in
            completion(completed)
        }
    }

    private func presentEmptyEmailConfirmationAlert(completion: @escaping (Bool) -> Void) {
        let message = NSLocalizedString(
            "EMPTY_EMAIL_ALERT_MESSAGE",
            tableName: "ProblemReport",
            comment: "Alert message warning users that they atttempt to send problem report without email address filled in."
        )

        let alertController = UIAlertController(title: nil, message: message, preferredStyle: .alert)

        let cancelAction = UIAlertAction(title: NSLocalizedString(
            "EMPTY_EMAIL_ALERT_CANCEL_ACTION",
            tableName: "ProblemReport",
            comment: "Cancel button."
        ), style: .cancel) { _ in
            completion(false)
        }
        let sendAction = UIAlertAction(title: NSLocalizedString(
            "EMPTY_EMAIL_ALERT_SEND_ANYWAY_ACTION",
            tableName: "ProblemReport",
            comment: "Title for button to send problem report without email address filled in."
        ), style: .destructive) { _ in
            completion(true)
        }

        alertController.addAction(cancelAction)
        alertController.addAction(sendAction)

        present(alertController, animated: true)
    }

    // MARK: - Private: Problem report submission

    private var showsSubmissionOverlay = false

    private func showSubmissionOverlay() {
        guard !showsSubmissionOverlay else { return }

        self.showsSubmissionOverlay = true

        view.addSubview(submissionOverlayView)

        NSLayoutConstraint.activate([
            submissionOverlayView.topAnchor.constraint(equalTo: view.safeAreaLayoutGuide.topAnchor),
            submissionOverlayView.leadingAnchor.constraint(equalTo: view.safeAreaLayoutGuide.leadingAnchor),
            submissionOverlayView.trailingAnchor.constraint(equalTo: view.safeAreaLayoutGuide.trailingAnchor),
            submissionOverlayView.bottomAnchor.constraint(equalTo: view.safeAreaLayoutGuide.bottomAnchor),
        ])

        UIView.transition(from: scrollView, to: submissionOverlayView, duration: 0.25, options: [.showHideTransitionViews, .transitionCrossDissolve]) { (success) in
            // success
        }
    }

    private func hideSubmissionOverlay() {
        guard showsSubmissionOverlay else { return }

        self.showsSubmissionOverlay = false

        UIView.transition(from: submissionOverlayView, to: scrollView, duration: 0.25, options: [.showHideTransitionViews, .transitionCrossDissolve]) { (success) in
            // success
            self.submissionOverlayView.removeFromSuperview()
        }
    }

    // MARK: - Data model

    private struct ViewModel {
        let email: String
        let message: String

        init() {
            email = ""
            message = ""
        }

        init(email: String, message: String) {
            self.email = email.trimmingCharacters(in: .whitespacesAndNewlines)
            self.message = message.trimmingCharacters(in: .whitespacesAndNewlines)
        }

        var isValid: Bool {
            return !message.isEmpty
        }
    }

    private static var persistentViewModel = ViewModel()

    private func loadPersistentViewModel() {
        emailTextField.text = Self.persistentViewModel.email
        messageTextView.text = Self.persistentViewModel.message

        validateForm()
    }

    private func updatePersistentViewModel() {
        Self.persistentViewModel = ViewModel(
            email: emailTextField.text ?? "",
            message: messageTextView.text
        )

        validateForm()
    }

    private func clearPersistentViewModel() {
        Self.persistentViewModel = ViewModel()
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

    private func didSendProblemReport(viewModel: ViewModel, result: Result<(), REST.Error>) {
        switch result {
        case .success:
            submissionOverlayView.state = .sent(viewModel.email)

            // Clear persistent view model upon successful submission
            clearPersistentViewModel()

        case .failure(let error):
            submissionOverlayView.state = .failure(error)
        }

        navigationItem.setHidesBackButton(false, animated: true)
    }

    // MARK: - Problem report submission helpers

    private func sendProblemReport() {
        let viewModel = Self.persistentViewModel

        let log = consolidatedLog.string
        let metadata = consolidatedLog.metadata.reduce(into: [:]) { (output, entry) in
            output[entry.key.rawValue] = entry.value
        }

        let request = REST.ProblemReportRequest(address: viewModel.email, message: viewModel.message, log: log, metadata: metadata)

        willSendProblemReport()

        REST.Client.shared.sendProblemReport(request)
            .receive(on: .main)
            .observe { completion in
                self.didSendProblemReport(viewModel: viewModel, result: completion.unwrappedValue!)
            }
    }

    // MARK: - Input fields' notifications

    @objc private func messageTextViewDidBeginEditing() {
        setDescriptionFieldExpanded(true)
    }

    @objc private func messageTextViewDidEndEditing() {
        setDescriptionFieldExpanded(false)
    }

    @objc private func messageTextViewDidChange() {
        updatePersistentViewModel()
    }

    @objc private func emailTextFieldDidChange() {
        updatePersistentViewModel()
    }

    // MARK: - UITextFieldDelegate

    func textFieldShouldReturn(_ textField: UITextField) -> Bool {
        messageTextView.becomeFirstResponder()
        return false
    }

    // MARK: - ConditionalNavigation

    func shouldPopNavigationItem(_ navigationItem: UINavigationItem, trigger: NavigationPopTrigger) -> Bool {
        switch trigger {
        case .interactiveGesture:
            // Disable swipe when editing
            return !emailTextField.isFirstResponder && !messageTextView.isFirstResponder

        case .backButton:
            // Dismiss the keyboard to fix a visual glitch when moving back to the previous controller
            view.endEditing(true)
            return true
        }
    }

}
