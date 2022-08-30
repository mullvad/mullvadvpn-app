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
    private static let dimmingViewOpacityWhenPresented = 0.5
    private var isPresented = false

    private let dimmingView: UIView = {
        let dimmingView = UIView()
        dimmingView.backgroundColor = .black
        return dimmingView
    }()

    override var shouldRemovePresentersView: Bool {
        return false
    }

    override func viewWillTransition(
        to size: CGSize,
        with coordinator: UIViewControllerTransitionCoordinator
    ) {
        super.viewWillTransition(to: size, with: coordinator)

        coordinator.animate { context in
            guard let containerView = self.containerView,
                  self.isPresented else { return }

            let targetFrame = FormsheetPresentationAnimator.targetFrame(
                in: containerView,
                preferredContentSize: self.presentedViewController.preferredContentSize
            )

            self.presentedView?.frame = targetFrame
        }
    }

    override func containerViewWillLayoutSubviews() {
        dimmingView.frame = containerView?.bounds ?? .zero
    }

    override func presentationTransitionWillBegin() {
        dimmingView.alpha = 0
        containerView?.addSubview(dimmingView)

        if let transitionCoordinator = presentingViewController.transitionCoordinator {
            transitionCoordinator.animate { context in
                self.dimmingView.alpha = Self.dimmingViewOpacityWhenPresented
            }
        } else {
            dimmingView.alpha = Self.dimmingViewOpacityWhenPresented
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
}
