//
//  FormsheetPresentationController.swift
//  MullvadVPN
//
//  Created by Andreas Lif on 2022-08-30.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

class FormsheetPresentationController: UIPresentationController {
    private let dimmingViewOpacityWhenPresented = 0.5
    private var isPresented = false

    private let dimmingView: UIView = {
        let dimmingView = UIView()
        dimmingView.backgroundColor = .black
        return dimmingView
    }()

    override var shouldRemovePresentersView: Bool {
        return false
    }

    override init(presentedViewController: UIViewController,
                  presenting presentingViewController: UIViewController?) {
        super.init(presentedViewController: presentedViewController,
                   presenting: presentingViewController)

        addKeyboardObservers()
    }

    override func viewWillTransition(to size: CGSize,
                                     with coordinator: UIViewControllerTransitionCoordinator) {
        super.viewWillTransition(to: size, with: coordinator)

        coordinator.animate { [weak self] context in
            guard let self = self,
                  let containerView = self.containerView,
                  self.isPresented else { return }

            let targetFrame = FormsheetPresentationAnimator
                .targetFrame(in: containerView,
                             preferredContentSize: CGSize(width: self.presentingViewController.view.frame.width - UIMetrics.contentLayoutMargins.left,
                                                          height: 300)
                )

            self.presentedViewController.view.frame = targetFrame
        }
    }

    override func containerViewWillLayoutSubviews() {
        dimmingView.frame = containerView?.bounds ?? .zero
    }

    override func presentationTransitionWillBegin() {
        dimmingView.alpha = 0
        containerView?.addSubview(dimmingView)

        if let transitionCoordinator = presentingViewController.transitionCoordinator {
            transitionCoordinator.animate { [weak self] context in
                guard let self = self else { return }
                self.dimmingView.alpha = self.dimmingViewOpacityWhenPresented
            }
        } else {
            dimmingView.alpha = self.dimmingViewOpacityWhenPresented
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

    deinit {
        removingKeyboardObservers()
    }
}

// MARK: - Keyboard delegates
// Putting most top view on the center of remaining height when keyboard opens.
private extension FormsheetPresentationController {
    /// Adding keyboard will show and will hide notification observers.
    private func addKeyboardObservers() {
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(keyboardWillShow),
            name: UIResponder.keyboardWillShowNotification,
            object: nil
        )
        
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(keyboardWillHide),
            name: UIResponder.keyboardWillHideNotification,
            object: nil
        )
    }

    /// Removing keyboard related observers.
    private func removingKeyboardObservers() {
        NotificationCenter.default.removeObserver(self,
                                                  name: UIResponder.keyboardWillChangeFrameNotification,
                                                  object: nil)
        NotificationCenter.default.removeObserver(self,
                                                  name: UIResponder.keyboardWillHideNotification,
                                                  object: nil)
    }

    /// Keyboard will show handling function, Puts presented view on the middle of remaining height.
    /// - Warning: Pins view to top if remaining height was not enough to fit the view.
    /// - Parameter notification: NSNotification that holds keyboard related info.
    @objc private func keyboardWillShow(_ notification: NSNotification) {
        guard let keyboardFrame = (notification
            .userInfo?[UIResponder.keyboardFrameEndUserInfoKey] as? NSValue)?.cgRectValue,
              let presentedView = presentedView
        else { return }

        let remainingHeight = keyboardFrame.origin.y / 2

        if remainingHeight > 0 {
            let center = CGPoint(x: presentedView.center.x, y: remainingHeight)
            presentedView.center = center
        } else {
            let topSafeAreaInset = presentingViewController.view.safeAreaInsets.top
            presentedView.frame.origin.y = topSafeAreaInset
        }
    }

    /// Keyboard will hide handling function, Puts presented view on middle of container view.
    ///  (Puts view on the place it was before opening keyboard)
    @objc private func keyboardWillHide() {
        guard let containerView = containerView,
              let presentedView = presentedView
        else { return }

        presentedView.center = containerView.center
    }
}
