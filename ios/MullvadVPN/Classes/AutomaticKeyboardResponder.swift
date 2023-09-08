//
//  AutomaticKeyboardResponder.swift
//  MullvadVPN
//
//  Created by pronebird on 24/03/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import UIKit

class AutomaticKeyboardResponder {
    weak var targetView: UIView?
    private let handler: (UIView, CGFloat) -> Void

    private var lastKeyboardRect: CGRect?

    private let logger = Logger(label: "AutomaticKeyboardResponder")
    private var presentationFrameObserver: NSKeyValueObservation?

    init<T: UIView>(targetView: T, handler: @escaping (T, CGFloat) -> Void) {
        self.targetView = targetView
        self.handler = { view, adjustment in
            if let view = view as? T {
                handler(view, adjustment)
            }
        }

        NotificationCenter.default.addObserver(
            self,
            selector: #selector(keyboardWillChangeFrame(_:)),
            name: UIResponder.keyboardWillChangeFrameNotification,
            object: nil
        )
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(keyboardWillShow(_:)),
            name: UIResponder.keyboardWillShowNotification,
            object: nil
        )
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(keyboardWillHide(_:)),
            name: UIResponder.keyboardWillHideNotification,
            object: nil
        )
    }

    func updateContentInsets() {
        guard let keyboardRect = lastKeyboardRect else { return }
        adjustContentInsets(convertedKeyboardFrameEnd: keyboardRect)
    }

    // MARK: - Keyboard notifications

    @objc private func keyboardWillShow(_ notification: Notification) {
        addPresentationControllerObserver()
    }

    @objc private func keyboardWillHide(_ notification: Notification) {
        presentationFrameObserver = nil
    }

    @objc private func keyboardWillChangeFrame(_ notification: Notification) {
        handleKeyboardNotification(notification)
    }

    // MARK: - Private

    private func handleKeyboardNotification(_ notification: Notification) {
        guard let userInfo = notification.userInfo,
              let targetView else { return }
        // In iOS 16.1 and later, the keyboard notification object is the screen the keyboard appears on.
        if #available(iOS 16.1, *) {
            guard let screen = notification.object as? UIScreen,
                  // Get the keyboard’s frame at the end of its animation.
                  let keyboardFrameEnd = userInfo[UIResponder.keyboardFrameEndUserInfoKey] as? CGRect else { return }

            // Use that screen to get the coordinate space to convert from.
            let fromCoordinateSpace = screen.coordinateSpace

            // Get your view's coordinate space.
            let toCoordinateSpace: UICoordinateSpace = targetView

            // Convert the keyboard's frame from the screen's coordinate space to your view's coordinate space.
            let convertedKeyboardFrameEnd = fromCoordinateSpace.convert(keyboardFrameEnd, to: toCoordinateSpace)

            lastKeyboardRect = convertedKeyboardFrameEnd

            adjustContentInsets(convertedKeyboardFrameEnd: convertedKeyboardFrameEnd)
        } else {
            guard let keyboardValue = notification.userInfo?[UIResponder.keyboardFrameEndUserInfoKey] as? NSValue
            else { return }
            let keyboardFrameEnd = keyboardValue.cgRectValue
            let convertedKeyboardFrameEnd = targetView.convert(keyboardFrameEnd, from: targetView.window)
            lastKeyboardRect = convertedKeyboardFrameEnd

            adjustContentInsets(convertedKeyboardFrameEnd: convertedKeyboardFrameEnd)
        }
    }

    private func addPresentationControllerObserver() {
        guard isFormSheetPresentation else { return }

        // Presentation controller follows the keyboard on iPad.
        // Install the observer to listen for the container view frame and adjust the target view
        // accordingly.
        guard let containerView = presentationContainerView else {
            logger.warning("Cannot determine the container view in form sheet presentation.")
            return
        }

        presentationFrameObserver = containerView.observe(
            \.frame,
            options: [.new],
            changeHandler: { [weak self] _, _ in
                guard let self,
                      let keyboardFrameValue = lastKeyboardRect else { return }

                adjustContentInsets(convertedKeyboardFrameEnd: keyboardFrameValue)
            }
        )
    }

    /// Returns the first parent controller in the responder chain
    private var parentViewController: UIViewController? {
        var responder: UIResponder? = targetView
        let iterator = AnyIterator { () -> UIResponder? in
            responder = responder?.next
            return responder
        }
        return iterator.first { $0 is UIViewController } as? UIViewController
    }

    /// Returns the presentation container view that's moved along with the keyboard on iPad
    private var presentationContainerView: UIView? {
        var currentView = parentViewController?.view
        let iterator = AnyIterator { () -> UIView? in
            currentView = currentView?.superview
            return currentView
        }

        // Find the container view that private `_UIFormSheetPresentationController` moves
        // along with the keyboard.
        return iterator.first { view -> Bool in
            view.description.starts(with: "<UIDropShadowView")
        }
    }

    private var isFormSheetPresentation: Bool {
        // Form sheet is only supported on iPad
        guard UIDevice.current.userInterfaceIdiom == .pad else { return false }

        // Find the parent controller holding the view
        guard let parent = parentViewController else { return false }

        // Determine presentation style within the context
        let presentationStyle: UIModalPresentationStyle

        // Use the presentation style of a presented controller,
        // when parent controller is being presented as a child of other modal controller.
        if let presented = parent.presentingViewController?.presentedViewController {
            presentationStyle = presented.modalPresentationStyle
        } else {
            presentationStyle = parent.modalPresentationStyle
        }

        return presentationStyle == .formSheet
    }

    private func adjustContentInsets(convertedKeyboardFrameEnd: CGRect) {
        guard let targetView else { return }

        // Get the safe area insets when the keyboard is offscreen.
        var bottomOffset = targetView.safeAreaInsets.bottom

        // Get the intersection between the keyboard's frame and the view's bounds to work with the
        // part of the keyboard that overlaps your view.
        let viewIntersection = targetView.bounds.intersection(convertedKeyboardFrameEnd)

        // Check whether the keyboard intersects your view before adjusting your offset.
        if !viewIntersection.isEmpty {
            // Adjust the offset by the difference between the view's height and the height of the
            // intersection rectangle.
            bottomOffset = targetView.bounds.maxY - viewIntersection.minY
        }

        handler(targetView, bottomOffset)
    }
}

extension AutomaticKeyboardResponder {
    /// A convenience initializer that automatically assigns the offset to the scroll view
    /// subclasses
    convenience init(targetView: some UIScrollView) {
        self.init(targetView: targetView) { scrollView, offset in
            if scrollView.canBecomeFirstResponder {
                scrollView.contentInset.bottom = targetView.isFirstResponder ? offset : 0
                scrollView.verticalScrollIndicatorInsets.bottom = targetView.isFirstResponder
                    ? offset
                    : 0
            } else {
                scrollView.contentInset.bottom = offset
                scrollView.verticalScrollIndicatorInsets.bottom = offset
            }
        }
    }
}
