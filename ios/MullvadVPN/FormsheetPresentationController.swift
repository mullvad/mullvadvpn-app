//
//  FormsheetPresentationController.swift
//  MullvadVPN
//
//  Created by Andreas Lif on 2022-08-30.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

final class FormsheetPresentationController: UIPresentationController {
    private let dimmingViewOpacityWhenPresented = 0.5
    private var isPresented = false
    private var keyboardFrame: CGRect = .zero

    private let dimmingView: UIView = {
        let dimmingView = UIView()
        dimmingView.backgroundColor = .black
        return dimmingView
    }()

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
        updatePresentedViewLayout()
    }

    override func presentationTransitionWillBegin() {
        dimmingView.alpha = 0
        presentedView?.layer.cornerRadius = 16
        containerView?.addSubview(dimmingView)

        if let transitionCoordinator = presentingViewController.transitionCoordinator {
            transitionCoordinator.animate { [weak self] context in
                guard let self = self else { return }

                self.dimmingView.alpha = self.dimmingViewOpacityWhenPresented
            }
        } else {
            dimmingView.alpha = dimmingViewOpacityWhenPresented
        }
    }

    override func presentationTransitionDidEnd(_ completed: Bool) {
        if completed {
            isPresented = true
        } else {
            dimmingView.removeFromSuperview()
        }
    }

    override func dismissalTransitionWillBegin() {
        presentingViewController.transitionCoordinator?.animate { context in
            self.dimmingView.alpha = 0
        }
    }

    override func dismissalTransitionDidEnd(_ completed: Bool) {
        if completed {
            dimmingView.removeFromSuperview()
            isPresented = false
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

        animateAlongsideKeyboard(notification: notification) {
            self.updatePresentedViewLayout()
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
        guard isPresented else { return }

        let targetFrame = FormsheetPresentationAnimator.targetFrame(
            in: availableContainerBounds,
            preferredContentSize: presentedViewController.preferredContentSize
        )

        presentedView?.frame = targetFrame
    }

    private var availableContainerBounds: CGRect {
        guard let containerView = containerView else { return .zero }

        let safeAreaBounds = containerView.safeAreaLayoutGuide.layoutFrame

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
