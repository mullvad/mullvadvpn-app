//
//  FormsheetPresentationController.swift
//  MullvadVPN
//
//  Created by pronebird on 2022-12-16.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import UIKit

final class FormsheetPresentationController: UIPresentationController {
    private static let dimmingViewOpacityWhenPresented = 0.5
    private static let shadowRadius: CGFloat = 8

    private var keyboardFrame: CGRect = .zero

    private let dimmingView: UIView = {
        let dimmingView = UIView()
        dimmingView.backgroundColor = .black
        return dimmingView
    }()

    private let shadowView: UIView = {
        let shadowView = UIView()
        shadowView.layer.shadowRadius = shadowRadius
        shadowView.layer.shadowOpacity = 0.35
        shadowView.layer.shadowColor = UIColor.black.cgColor
        return shadowView
    }()

    var initialFrameOfPresentedViewInContainerView: CGRect {
        guard let containerView = containerView else { return .zero }

        var frame = frameOfPresentedViewInContainerView
        frame.origin.y = containerView.bounds.maxY + Self.shadowRadius

        return frame
    }

    override var frameOfPresentedViewInContainerView: CGRect {
        let containerBounds = availableContainerBounds
        let preferredContentSize = presentedViewController.preferredContentSize

        let presentedViewSize = CGSize(
            width: min(containerBounds.size.width, preferredContentSize.width),
            height: min(containerBounds.size.height, preferredContentSize.height)
        )

        return CGRect(
            origin: CGPoint(
                x: containerBounds.midX - presentedViewSize.width * 0.5,
                y: containerBounds.midY - presentedViewSize.height * 0.5
            ),
            size: presentedViewSize
        )
    }

    override var shouldRemovePresentersView: Bool {
        return false
    }

    override init(
        presentedViewController: UIViewController,
        presenting presentingViewController: UIViewController?
    ) {
        super.init(
            presentedViewController: presentedViewController,
            presenting: presentingViewController
        )

        addKeyboardObserver()
    }

    override func containerViewWillLayoutSubviews() {
        super.containerViewWillLayoutSubviews()

        dimmingView.frame = containerView?.bounds ?? .zero

        // TODO: handle keyboard during rotation.
        // Keyboard notifications arrive after layout pass during interface rotation, which makes it
        // impossible to handle rotation properly as we don't know the keyboard frame until later.
        updatePresentedViewLayout()
        updateShadowLayout()
    }

    override func presentationTransitionWillBegin() {
        dimmingView.alpha = 0

        presentedView?.clipsToBounds = true
        presentedView?.layer.cornerRadius = 8

        containerView?.addSubview(dimmingView)
        containerView?.addSubview(shadowView)

        var shadowFrame = frameOfPresentedViewInContainerView
        shadowFrame.origin.y = containerView?.bounds.maxY ?? 0
        shadowView.frame = shadowFrame

        let animations = {
            self.dimmingView.alpha = Self.dimmingViewOpacityWhenPresented
            self.updateShadowLayout()
        }

        if let transitionCoordinator = presentingViewController.transitionCoordinator {
            transitionCoordinator.animate { context in
                animations()
            }
        } else {
            animations()
        }
    }

    override func presentationTransitionDidEnd(_ completed: Bool) {
        if !completed {
            dimmingView.removeFromSuperview()
            shadowView.removeFromSuperview()
        }
    }

    override func dismissalTransitionWillBegin() {
        presentingViewController.transitionCoordinator?.animate { context in
            self.dimmingView.alpha = 0
            self.updateShadowLayout()
        }
    }

    override func dismissalTransitionDidEnd(_ completed: Bool) {
        if completed {
            dimmingView.removeFromSuperview()
            shadowView.removeFromSuperview()
        }
    }

    private func addKeyboardObserver() {
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(keyboardWillChangeFrame),
            name: UIResponder.keyboardWillChangeFrameNotification,
            object: nil
        )
    }

    @objc private func keyboardWillChangeFrame(_ notification: Notification) {
        guard let keyboardFrameValue = notification
            .userInfo?[UIResponder.keyboardFrameEndUserInfoKey] as? NSValue else { return }

        let keyboardEndFrame = keyboardFrameValue.cgRectValue

        keyboardFrame = isKeyboardDocked(keyboardEndFrame) && isKeyboardVisible(keyboardEndFrame)
            ? keyboardEndFrame
            : .zero

        // Ignore keyboard during dismissal.
        guard !presentedViewController.isBeingDismissed else { return }

        // Ignore keyboard presented controller is offscreen.
        guard presentedViewController.viewIfLoaded?.superview != nil else { return }

        animateAlongsideKeyboard(notification: notification) {
            self.updatePresentedViewLayout()
            self.updateShadowLayout()
        }
    }

    private func animateAlongsideKeyboard(
        notification: Notification,
        animations: @escaping () -> Void
    ) {
        let animationCurveValue = notification
            .userInfo?[UIResponder.keyboardAnimationCurveUserInfoKey] as? NSNumber
        let animationDurationValue = notification
            .userInfo?[UIResponder.keyboardAnimationDurationUserInfoKey] as? NSNumber

        guard let animationCurveValue = animationCurveValue,
              let animationDuration = animationDurationValue?.doubleValue,
              let animationCurve = UIView.AnimationCurve(rawValue: animationCurveValue.intValue)
        else {
            animations()
            return
        }

        let animator = UIViewPropertyAnimator(
            duration: animationDuration,
            curve: animationCurve,
            animations: animations
        )

        animator.startAnimation()
    }

    private func isKeyboardVisible(_ keyboardFrame: CGRect) -> Bool {
        guard let screenBounds = containerView?.window?.screen.bounds else { return false }

        return keyboardFrame.intersects(screenBounds)
    }

    private func isKeyboardDocked(_ keyboardFrame: CGRect) -> Bool {
        guard let screenBounds = containerView?.window?.screen.bounds else { return false }

        return keyboardFrame.minX == screenBounds.minX &&
            keyboardFrame.maxX == screenBounds.maxX &&
            keyboardFrame.maxY == screenBounds.maxY
    }

    private func updatePresentedViewLayout() {
        presentedView?.frame = frameOfPresentedViewInContainerView
    }

    private func updateShadowLayout() {
        var presentedViewFrame = frameOfPresentedViewInContainerView

        if presentedViewController.isBeingDismissed {
            presentedViewFrame.origin.y = containerView?.bounds.maxY ?? 0
        }

        shadowView.frame = presentedViewFrame
        shadowView.layer.shadowPath = UIBezierPath(
            rect: CGRect(origin: .zero, size: presentedViewFrame.size)
        ).cgPath
    }

    private var availableContainerBounds: CGRect {
        guard let containerView = containerView else { return .zero }

        var safeAreaBounds = containerView.safeAreaLayoutGuide.layoutFrame
        safeAreaBounds.origin.x = max(safeAreaBounds.origin.x, UIMetrics.contentLayoutMargins.left)
        safeAreaBounds.size.width = min(
            safeAreaBounds.size.width,
            containerView.bounds.width - UIMetrics.contentLayoutMargins.left
                - UIMetrics.contentLayoutMargins.right
        )

        guard keyboardFrame != .zero else { return safeAreaBounds }

        let containerFrame = containerView.convert(safeAreaBounds, to: nil)
        let intersectionRect = containerFrame.intersection(keyboardFrame)

        guard !CGRectIsInfinite(intersectionRect) else { return safeAreaBounds }

        return CGRect(
            origin: safeAreaBounds.origin,
            size: CGSize(
                width: containerFrame.width,
                height: containerFrame.height - intersectionRect.height
            )
        )
    }
}
