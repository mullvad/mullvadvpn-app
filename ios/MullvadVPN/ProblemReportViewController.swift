//
//  ProblemReportViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 15/09/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import UIKit

class ProblemReportViewController: UIViewController, UITextFieldDelegate, ConditionalNavigation {

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
        containerView.layoutMargins = UIEdgeInsets(top: 8, left: 24, bottom: 24, right: 24)
        containerView.backgroundColor = .clear
        return containerView
    }()

    /// Subheading label displayed below navigation bar
    private lazy var subheaderLabel: UILabel = {
        let textLabel = UILabel()
        textLabel.translatesAutoresizingMaskIntoConstraints = false
        textLabel.numberOfLines = 0
        textLabel.textColor = .white
        textLabel.text = NSLocalizedString("To help you more effectively, your app's log file will be attached to this message. Your data will remain secure and private, as it is anonymised before being sent over an encrypted channel.", comment: "")

        return textLabel
    }()

    /// Email text field
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
        textField.placeholder = NSLocalizedString("Your email (optional)", comment: "")

        return textField
    }()

    /// Description text view
    private lazy var descriptionTextView: CustomTextView = {
        let textView = CustomTextView()
        textView.translatesAutoresizingMaskIntoConstraints = false
        textView.backgroundColor = .white
        textView.inputAccessoryView = descriptionAccessoryToolbar
        textView.font = UIFont.systemFont(ofSize: 17)
        textView.placeholder = NSLocalizedString("Describe your problem", comment: "")
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
    private var activeDescriptionTextViewConstraints = [NSLayoutConstraint]()

    /// Constraints used when description text view is inactive
    private var inactiveDescriptionTextViewConstraints = [NSLayoutConstraint]()

    /// Flag indicating when the text view is expanded to fill the entire view
    private var descriptionTextViewExpanded = false

    /// Keyboard intersection with the controller view
    private var keyboardIntersectionRect = CGRect.zero

    /// Bottom content inset necessary to compensate for the keyboard overlapping
    var scrollViewBottomContentInsetAccountingForKeyboard: CGFloat {
        return max(0, keyboardIntersectionRect.height - view.safeAreaInsets.bottom)
    }

    /// Placeholder view used to fill the space within the scroll view when the text view is
    /// expanded to fill the entire view
    private lazy var descriptionPlaceholder: UIView = {
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
        button.setTitle(NSLocalizedString("View app logs", comment: ""), for: .normal)
        button.addTarget(self, action: #selector(handleViewLogsButtonTap), for: .touchUpInside)
        return button
    }()

    private lazy var sendButton: AppButton = {
        let button = AppButton(style: .success)
        button.translatesAutoresizingMaskIntoConstraints = false
        button.setTitle(NSLocalizedString("Send", comment: ""), for: .normal)
        button.addTarget(self, action: #selector(handleSendButtonTap), for: .touchUpInside)
        return button
    }()

    private lazy var emailAccessoryToolbar: UIToolbar = {
        return makeKeyboardToolbar(canGoBackward: false, canGoForward: true)
    }()

    private lazy var descriptionAccessoryToolbar: UIToolbar = {
        return makeKeyboardToolbar(canGoBackward: true, canGoForward: false)
    }()

    override func viewDidLoad() {
        super.viewDidLoad()

        view.backgroundColor = .secondaryColor

        navigationItem.title = NSLocalizedString("Report a problem", comment: "Navigation title")

        // Make sure that the user can't easily dismiss the controller on iOS 13 and above
        if #available(iOS 13.0, *) {
            isModalInPresentation = true
        }

        // Set hugging & compression priorities so that description text view wants to grow
        emailTextField.setContentHuggingPriority(.defaultHigh, for: .vertical)
        emailTextField.setContentCompressionResistancePriority(.defaultHigh, for: .vertical)
        descriptionTextView.setContentHuggingPriority(.defaultLow, for: .vertical)
        descriptionTextView.setContentCompressionResistancePriority(.defaultLow, for: .vertical)

        textFieldsHolder.addSubview(emailTextField)
        textFieldsHolder.addSubview(descriptionPlaceholder)
        textFieldsHolder.addSubview(descriptionTextView)

        view.addSubview(scrollView)
        scrollView.addSubview(containerView)
        containerView.addSubview(subheaderLabel)
        containerView.addSubview(textFieldsHolder)
        containerView.addSubview(buttonsStackView)

        addConstraints()

        registerKeyboardNotifications()
        registerDescriptionTextViewNotifications()
    }

    // MARK: - Actions

    @objc func focusEmailTextField() {
        emailTextField.becomeFirstResponder()
    }

    @objc func focusDescriptionTextView() {
        descriptionTextView.becomeFirstResponder()
    }

    @objc func dismissKeyboard() {
        view.endEditing(false)
    }

    @objc func handleSendButtonTap() {
        // TODO: implement
    }

    @objc func handleViewLogsButtonTap() {
        // TODO: implement
    }

    // MARK: - Private

    private func registerKeyboardNotifications() {
        NotificationCenter.default.addObserver(self, selector: #selector(keyboardWillChangeFrame(_:)),
                                               name: UIWindow.keyboardWillChangeFrameNotification,
                                               object: nil)
    }

    private func registerDescriptionTextViewNotifications() {
        let notificationCenter = NotificationCenter.default

        notificationCenter.addObserver(forName: UITextView.textDidBeginEditingNotification, object: descriptionTextView, queue: OperationQueue.main) { [weak self] (note) in
            // begin editing
            self?.setDescriptionFieldExpanded(true)
        }
        notificationCenter.addObserver(forName: UITextView.textDidEndEditingNotification, object: descriptionTextView, queue: OperationQueue.main) { [weak self] (note) in
            // begin editing
            self?.setDescriptionFieldExpanded(false)
        }
    }

    private func makeKeyboardToolbar(canGoBackward: Bool, canGoForward: Bool) -> UIToolbar {
        var toolbarItems = makeKeyboardNavigationItems(canGoBackward: canGoBackward, canGoForward: canGoForward)

        toolbarItems.append(contentsOf: [
            UIBarButtonItem(barButtonSystemItem: .flexibleSpace, target: nil, action: nil),
            UIBarButtonItem(barButtonSystemItem: .done, target: self, action: #selector(dismissKeyboard))
        ])

        let toolbar = UIToolbar(frame: CGRect(x: 0, y: 0, width: 100, height: 44))
        toolbar.items = toolbarItems
        return toolbar
    }

    private func makeKeyboardNavigationItems(canGoBackward: Bool, canGoForward: Bool) -> [UIBarButtonItem] {
        let prevButton: UIBarButtonItem
        let nextButton: UIBarButtonItem

        if #available(iOS 13, *) {
            prevButton = UIBarButtonItem(image: UIImage(systemName: "chevron.up"), style: .plain, target: self, action: #selector(focusEmailTextField))
        } else {
            prevButton = UIBarButtonItem(title: NSLocalizedString("Previous", comment: ""), style: .plain, target: self, action: #selector(focusEmailTextField))
        }

        prevButton.accessibilityLabel = NSLocalizedString("Previous", comment: "")
        prevButton.isEnabled = canGoBackward

        if #available(iOS 13, *) {
            nextButton = UIBarButtonItem(image: UIImage(systemName: "chevron.down"), style: .plain, target: self, action: #selector(focusDescriptionTextView))
        } else {
            nextButton = UIBarButtonItem(title: NSLocalizedString("Next", comment: ""), style: .plain, target: self, action: #selector(focusDescriptionTextView))
        }

        nextButton.accessibilityLabel = NSLocalizedString("Next", comment: "")
        nextButton.isEnabled = canGoForward

        if #available(iOS 13, *) {
            let spacer = UIBarButtonItem(barButtonSystemItem: .fixedSpace, target: nil, action: nil)
            spacer.width = 8
            return [prevButton, spacer, nextButton]
        } else {
            return [prevButton, nextButton]
        }
    }

    private func addConstraints() {
        self.activeDescriptionTextViewConstraints = [
            descriptionTextView.topAnchor.constraint(equalTo: view.topAnchor),
            descriptionTextView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            descriptionTextView.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            descriptionTextView.bottomAnchor.constraint(equalTo: view.bottomAnchor),
        ]

        self.inactiveDescriptionTextViewConstraints = [
            descriptionTextView.topAnchor.constraint(equalTo: emailTextField.bottomAnchor, constant: 12),
            descriptionTextView.leadingAnchor.constraint(equalTo: textFieldsHolder.leadingAnchor),
            descriptionTextView.trailingAnchor.constraint(equalTo: textFieldsHolder.trailingAnchor),
            descriptionTextView.bottomAnchor.constraint(equalTo: textFieldsHolder.bottomAnchor),
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

            descriptionPlaceholder.topAnchor.constraint(equalTo: emailTextField.bottomAnchor, constant: 12),
            descriptionPlaceholder.leadingAnchor.constraint(equalTo: textFieldsHolder.leadingAnchor),
            descriptionPlaceholder.trailingAnchor.constraint(equalTo: textFieldsHolder.trailingAnchor),
            descriptionPlaceholder.bottomAnchor.constraint(equalTo: textFieldsHolder.bottomAnchor),
            descriptionPlaceholder.heightAnchor.constraint(equalTo: descriptionTextView.heightAnchor),

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

            descriptionTextView.heightAnchor.constraint(greaterThanOrEqualToConstant: 150),
        ]

        constraints.append(contentsOf: self.inactiveDescriptionTextViewConstraints)

        NSLayoutConstraint.activate(constraints)
    }

    private func setDescriptionFieldExpanded(_ isExpanded: Bool) {
        if isExpanded {
            // Disable the large title
            self.navigationItem.largeTitleDisplayMode = .never

            // Move the text view above scroll view
            view.addSubview(self.descriptionTextView)

            // Re-add old constraints
            NSLayoutConstraint.activate(self.inactiveDescriptionTextViewConstraints)

            // Do a layout pass
            view.layoutIfNeeded()

            // Swap constraints
            NSLayoutConstraint.deactivate(self.inactiveDescriptionTextViewConstraints)
            NSLayoutConstraint.activate(self.activeDescriptionTextViewConstraints)

            // Enable content inset adjustment on text view
            self.descriptionTextView.contentInsetAdjustmentBehavior = .always

            // Animate constraints & rounded corners on the text view
            animateDescriptionTextView(animations: {
                // Turn off rounded corners as the text view fills in the entire view
                self.descriptionTextView.roundCorners = false

                self.view.layoutIfNeeded()

            }) { (completed) in
                // no-op
                self.descriptionTextViewExpanded = true

                self.updateDescriptionTextViewContentInsets()
            }

        } else {
            // Re-enable the large title
            self.navigationItem.largeTitleDisplayMode = .automatic

            // Swap constraints
            NSLayoutConstraint.deactivate(self.activeDescriptionTextViewConstraints)
            NSLayoutConstraint.activate(self.inactiveDescriptionTextViewConstraints)

            // Animate constraints & rounded corners on the text view
            animateDescriptionTextView(animations: {
                // Turn on rounded corners as the text view returns back to where it was
                self.descriptionTextView.roundCorners = true

                self.view.layoutIfNeeded()
            }) { (completed) in
                // Revert the content adjustment behavior
                self.descriptionTextView.contentInsetAdjustmentBehavior = .never

                // Add the text view inside of the scroll view
                self.textFieldsHolder.addSubview(self.descriptionTextView)

                self.descriptionTextViewExpanded = false
            }
        }
    }

    private func updateScrollViewContentInsets() {
        let scrollViewBottomInset = scrollViewBottomContentInsetAccountingForKeyboard

        scrollView.contentInset.bottom = scrollViewBottomInset
        scrollView.scrollIndicatorInsets.bottom = scrollViewBottomInset
    }

    private func updateDescriptionTextViewContentInsets() {
        // Ignore updating text view insets until it's fully expanded
        guard descriptionTextViewExpanded else { return }

        let textViewBottomInset: CGFloat

        if descriptionTextView.isFirstResponder {
            textViewBottomInset = scrollViewBottomContentInsetAccountingForKeyboard
        } else {
            textViewBottomInset = 0
        }

        descriptionTextView.contentInset.bottom = textViewBottomInset
        descriptionTextView.scrollIndicatorInsets.bottom = textViewBottomInset
    }

    override func viewSafeAreaInsetsDidChange() {
        super.viewSafeAreaInsetsDidChange()

        updateScrollViewContentInsets()
        updateDescriptionTextViewContentInsets()
    }

    private func animateDescriptionTextView(animations: @escaping () -> Void, completion: @escaping (Bool) -> Void) {
        UIView.animate(withDuration: 0.25, animations: animations) { (completed) in
            completion(completed)
        }
    }

    // MARK: - Keyboard notifications

    @objc private func keyboardWillChangeFrame(_ notification: Notification) {
        guard let keyboardFrameValue = notification.userInfo?[UIWindow.keyboardFrameEndUserInfoKey] as? NSValue else { return }

        let screenRect = self.view.convert(self.view.bounds, to: nil)

        keyboardIntersectionRect = screenRect.intersection(keyboardFrameValue.cgRectValue)

        updateScrollViewContentInsets()
        updateDescriptionTextViewContentInsets()
    }

    // MARK: - UITextFieldDelegate

    func textFieldShouldReturn(_ textField: UITextField) -> Bool {
        descriptionTextView.becomeFirstResponder()
        return false
    }

    // MARK: - ConditionalNavigation

    func shouldPopNavigationItem(_ navigationItem: UINavigationItem, trigger: NavigationPopTrigger) -> Bool {
        switch trigger {
        case .interactiveGesture:
            // Disable swipe when editing
            return !emailTextField.isFirstResponder && !descriptionTextView.isFirstResponder

        case .backButton:
            // Dismiss the keyboard to fix some visual glitching
            view.endEditing(true)
            return true
        }
    }

}
