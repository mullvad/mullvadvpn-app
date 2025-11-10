//
//  AutomaticKeyboardResponder.swift
//  MullvadVPN
//
//  Created by pronebird on 24/03/2021.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import UIKit

@MainActor
class AutomaticKeyboardResponder {
    weak var targetView: UIView?
    private let handler: (UIView, CGFloat) -> Void

    private var lastKeyboardRect: CGRect?

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
    }

    func updateContentInsets() {
        guard let keyboardRect = lastKeyboardRect else { return }
        adjustContentInsets(convertedKeyboardFrameEnd: keyboardRect)
    }

    // MARK: - Keyboard notifications

    @objc private func keyboardWillChangeFrame(_ notification: Notification) {
        handleKeyboardNotification(notification)
    }

    // MARK: - Private

    private func handleKeyboardNotification(_ notification: Notification) {
        guard let userInfo = notification.userInfo,
            let targetView
        else { return }
        guard let screen = notification.object as? UIScreen,
            // Get the keyboard’s frame at the end of its animation.
            let keyboardFrameEnd = userInfo[UIResponder.keyboardFrameEndUserInfoKey] as? CGRect
        else { return }

        // Use that screen to get the coordinate space to convert from.
        let fromCoordinateSpace = screen.coordinateSpace

        // Get your view's coordinate space.
        let toCoordinateSpace: UICoordinateSpace = targetView

        // Convert the keyboard's frame from the screen's coordinate space to your view's coordinate space.
        let convertedKeyboardFrameEnd = fromCoordinateSpace.convert(keyboardFrameEnd, to: toCoordinateSpace)

        lastKeyboardRect = convertedKeyboardFrameEnd

        adjustContentInsets(convertedKeyboardFrameEnd: convertedKeyboardFrameEnd)
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
                scrollView.verticalScrollIndicatorInsets.bottom =
                    targetView.isFirstResponder
                    ? offset
                    : 0
            } else {
                scrollView.contentInset.bottom = offset
                scrollView.verticalScrollIndicatorInsets.bottom = offset
            }
        }
    }
}
