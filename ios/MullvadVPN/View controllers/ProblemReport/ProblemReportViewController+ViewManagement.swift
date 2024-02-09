//
//  ProblemReportViewController+ViewManagement.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2024-02-09.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

extension ProblemReportViewController {
    func makeScrollView() -> UIScrollView {
        let scrollView = UIScrollView()
        scrollView.translatesAutoresizingMaskIntoConstraints = false
        scrollView.backgroundColor = .clear
        return scrollView
    }

    func makeContainerView() -> UIView {
        let containerView = UIView()
        containerView.translatesAutoresizingMaskIntoConstraints = false
        containerView.directionalLayoutMargins = UIMetrics.contentLayoutMargins
        containerView.backgroundColor = .clear
        return containerView
    }

    func makeSubheaderLabel() -> UILabel {
        let textLabel = UILabel()
        textLabel.translatesAutoresizingMaskIntoConstraints = false
        textLabel.numberOfLines = 0
        textLabel.textColor = .white
        textLabel.text = Self.persistentViewModel.subheadLabelText
        return textLabel
    }

    func makeEmailTextField() -> CustomTextField {
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
        textField.placeholder = Self.persistentViewModel.emailPlaceholderText
        return textField
    }

    func makeMessageTextView() -> CustomTextView {
        let textView = CustomTextView()
        textView.translatesAutoresizingMaskIntoConstraints = false
        textView.backgroundColor = .white
        textView.inputAccessoryView = messageAccessoryToolbar
        textView.font = UIFont.systemFont(ofSize: 17)
        textView.placeholder = Self.persistentViewModel.messageTextViewPlaceholder
        textView.contentInsetAdjustmentBehavior = .never

        return textView
    }

    func makeTextFieldsHolder() -> UIView {
        let view = UIView()
        view.translatesAutoresizingMaskIntoConstraints = false
        return view
    }

    func makeMessagePlaceholderView() -> UIView {
        let view = UIView()
        view.translatesAutoresizingMaskIntoConstraints = false
        view.backgroundColor = .clear
        return view
    }

    func makeButtonsStackView() -> UIStackView {
        let stackView = UIStackView(arrangedSubviews: [self.viewLogsButton, self.sendButton])
        stackView.translatesAutoresizingMaskIntoConstraints = false
        stackView.axis = .vertical
        stackView.spacing = 18

        return stackView
    }

    func makeViewLogsButton() -> AppButton {
        let button = AppButton(style: .default)
        button.translatesAutoresizingMaskIntoConstraints = false
        button.setTitle(Self.persistentViewModel.viewLogsButtonTitle, for: .normal)
        button.addTarget(self, action: #selector(handleViewLogsButtonTap), for: .touchUpInside)
        return button
    }

    func makeSendButton() -> AppButton {
        let button = AppButton(style: .success)
        button.translatesAutoresizingMaskIntoConstraints = false
        button.setTitle(Self.persistentViewModel.sendLogsButtonTitle, for: .normal)
        button.addTarget(self, action: #selector(handleSendButtonTap), for: .touchUpInside)
        return button
    }

    func makeSubmissionOverlayView() -> ProblemReportSubmissionOverlayView {
        let overlay = ProblemReportSubmissionOverlayView()
        overlay.translatesAutoresizingMaskIntoConstraints = false

        overlay.editButtonAction = { [weak self] in
            self?.hideSubmissionOverlay()
        }

        overlay.retryButtonAction = { [weak self] in
            self?.sendProblemReport()
        }

        return overlay
    }

    func addConstraints() {
        activeMessageTextViewConstraints =
            messageTextView.pinEdges(.all().excluding(.top), to: view) +
            messageTextView.pinEdges(PinnableEdges([.top(0)]), to: view.safeAreaLayoutGuide)

        inactiveMessageTextViewConstraints =
            messageTextView.pinEdges(.all().excluding(.top), to: textFieldsHolder) +
            [messageTextView.topAnchor.constraint(equalTo: emailTextField.bottomAnchor, constant: 12)]

        textFieldsHolder.addSubview(emailTextField)
        textFieldsHolder.addSubview(messagePlaceholder)
        textFieldsHolder.addSubview(messageTextView)

        scrollView.addSubview(containerView)
        containerView.addSubview(subheaderLabel)
        containerView.addSubview(textFieldsHolder)
        containerView.addSubview(buttonsStackView)

        view.addConstrainedSubviews([scrollView]) {
            inactiveMessageTextViewConstraints

            subheaderLabel.pinEdges(.all().excluding(.bottom), to: containerView.layoutMarginsGuide)

            textFieldsHolder.pinEdges(PinnableEdges([.leading(0), .trailing(0)]), to: containerView.layoutMarginsGuide)
            textFieldsHolder.topAnchor.constraint(equalTo: subheaderLabel.bottomAnchor, constant: 24)

            buttonsStackView.pinEdges(.all().excluding(.top), to: containerView.layoutMarginsGuide)
            buttonsStackView.topAnchor.constraint(equalTo: textFieldsHolder.bottomAnchor, constant: 18)

            emailTextField.pinEdges(.all().excluding(.bottom), to: textFieldsHolder)

            messagePlaceholder.pinEdges(.all().excluding(.top), to: textFieldsHolder)
            messagePlaceholder.topAnchor.constraint(equalTo: emailTextField.bottomAnchor, constant: 12)
            messagePlaceholder.heightAnchor.constraint(equalTo: messageTextView.heightAnchor)

            scrollView.frameLayoutGuide.topAnchor.constraint(equalTo: view.topAnchor)
            scrollView.frameLayoutGuide.bottomAnchor.constraint(equalTo: view.bottomAnchor)
            scrollView.frameLayoutGuide.leadingAnchor.constraint(equalTo: view.leadingAnchor)
            scrollView.frameLayoutGuide.trailingAnchor.constraint(equalTo: view.trailingAnchor)

            scrollView.contentLayoutGuide.topAnchor.constraint(equalTo: containerView.topAnchor)
            scrollView.contentLayoutGuide.bottomAnchor.constraint(equalTo: containerView.bottomAnchor)
            scrollView.contentLayoutGuide.leadingAnchor.constraint(equalTo: containerView.leadingAnchor)
            scrollView.contentLayoutGuide.trailingAnchor.constraint(equalTo: containerView.trailingAnchor)
            scrollView.contentLayoutGuide.widthAnchor.constraint(equalTo: scrollView.frameLayoutGuide.widthAnchor)
            scrollView.contentLayoutGuide.heightAnchor
                .constraint(greaterThanOrEqualTo: scrollView.safeAreaLayoutGuide.heightAnchor)

            messageTextView.heightAnchor.constraint(greaterThanOrEqualToConstant: 150)
        }
    }

    override func viewSafeAreaInsetsDidChange() {
        super.viewSafeAreaInsetsDidChange()

        scrollViewKeyboardResponder?.updateContentInsets()
        textViewKeyboardResponder?.updateContentInsets()
    }

    func makeKeyboardToolbar(canGoBackward: Bool, canGoForward: Bool) -> UIToolbar {
        var toolbarItems = UIBarButtonItem.makeKeyboardNavigationItems { prevButton, nextButton in
            prevButton.target = self
            prevButton.action = #selector(focusEmailTextField)
            prevButton.isEnabled = canGoBackward

            nextButton.target = self
            nextButton.action = #selector(focusDescriptionTextView)
            nextButton.isEnabled = canGoForward
        }

        toolbarItems.append(contentsOf: [
            UIBarButtonItem(barButtonSystemItem: .flexibleSpace, target: nil, action: nil),
            UIBarButtonItem(
                barButtonSystemItem: .done,
                target: self,
                action: #selector(dismissKeyboard)
            ),
        ])

        let toolbar = UIToolbar(frame: CGRect(x: 0, y: 0, width: 100, height: 44))
        toolbar.items = toolbarItems
        return toolbar
    }

    func setDescriptionFieldExpanded(_ isExpanded: Bool) {
        // Make voice over ignore siblings when expanded
        messageTextView.accessibilityViewIsModal = isExpanded

        if isExpanded {
            // Disable the large title
            navigationItem.largeTitleDisplayMode = .never

            // Move the text view above scroll view
            view.addSubview(messageTextView)

            // Re-add old constraints
            NSLayoutConstraint.activate(inactiveMessageTextViewConstraints)

            // Do a layout pass
            view.layoutIfNeeded()

            // Swap constraints
            NSLayoutConstraint.deactivate(inactiveMessageTextViewConstraints)
            NSLayoutConstraint.activate(activeMessageTextViewConstraints)

            // Enable content inset adjustment on text view
            messageTextView.contentInsetAdjustmentBehavior = .always

            // Animate constraints & rounded corners on the text view
            animateDescriptionTextView(animations: {
                // Turn off rounded corners as the text view fills in the entire view
                self.messageTextView.roundCorners = false

                self.view.layoutIfNeeded()
            }, completion: { _ in
                self.isMessageTextViewExpanded = true

                self.textViewKeyboardResponder?.updateContentInsets()

                // Tell accessibility engine to scan the new layout
                UIAccessibility.post(notification: .layoutChanged, argument: nil)
            })

        } else {
            // Re-enable the large title
            navigationItem.largeTitleDisplayMode = .automatic

            // Swap constraints
            NSLayoutConstraint.deactivate(activeMessageTextViewConstraints)
            NSLayoutConstraint.activate(inactiveMessageTextViewConstraints)

            // Animate constraints & rounded corners on the text view
            animateDescriptionTextView(animations: {
                // Turn on rounded corners as the text view returns back to where it was
                self.messageTextView.roundCorners = true

                self.view.layoutIfNeeded()
            }, completion: { _ in
                // Revert the content adjustment behavior
                self.messageTextView.contentInsetAdjustmentBehavior = .never

                // Add the text view inside of the scroll view
                self.textFieldsHolder.addSubview(self.messageTextView)

                self.isMessageTextViewExpanded = false

                // Tell accessibility engine to scan the new layout
                UIAccessibility.post(notification: .layoutChanged, argument: nil)
            })
        }
    }

    func animateDescriptionTextView(
        animations: @escaping () -> Void,
        completion: @escaping (Bool) -> Void
    ) {
        UIView.animate(withDuration: 0.25, animations: animations) { completed in
            completion(completed)
        }
    }

    func showSubmissionOverlay() {
        guard !showsSubmissionOverlay else { return }

        showsSubmissionOverlay = true

        view.addSubview(submissionOverlayView)

        NSLayoutConstraint.activate([
            submissionOverlayView.topAnchor.constraint(equalTo: view.safeAreaLayoutGuide.topAnchor),
            submissionOverlayView.leadingAnchor
                .constraint(equalTo: view.safeAreaLayoutGuide.leadingAnchor),
            submissionOverlayView.trailingAnchor
                .constraint(equalTo: view.safeAreaLayoutGuide.trailingAnchor),
            submissionOverlayView.bottomAnchor
                .constraint(equalTo: view.safeAreaLayoutGuide.bottomAnchor),
        ])

        UIView.transition(
            from: scrollView,
            to: submissionOverlayView,
            duration: 0.25,
            options: [.showHideTransitionViews, .transitionCrossDissolve]
        ) { _ in
            // success
        }
    }

    func hideSubmissionOverlay() {
        guard showsSubmissionOverlay else { return }

        showsSubmissionOverlay = false

        UIView.transition(
            from: submissionOverlayView,
            to: scrollView,
            duration: 0.25,
            options: [.showHideTransitionViews, .transitionCrossDissolve]
        ) { _ in
            // success
            self.submissionOverlayView.removeFromSuperview()
        }
    }
}
