//
//  FormsheetPresentationController.swift
//  MullvadVPN
//
//  Created by pronebird on 18/02/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit
/**
 Custom implementation of a formsheet presentation controller.
 */
class FormSheetPresentationController: UIPresentationController {
    /**
     Name of notification posted when fullscreen presentation changes, including during initial presentation.
     */
    static let willChangeFullScreenPresentation = Notification
        .Name(rawValue: "FormsheetPresentationControllerWillChangeFullScreenPresentation")

    /**
     User info key passed along with `willChangeFullScreenPresentation` notification that contains boolean value that
     indicates if presentation controller is using fullscreen presentation.
     */
    static let isFullScreenUserInfoKey = "isFullScreen"

    /**
     Last known presentation style used to prevent emitting duplicate `willChangeFullScreenPresentation` notifications.
     */
    private var lastKnownIsInFullScreen: Bool?

    private var keyboardResponder: AutomaticKeyboardResponder?

    private let dimmingView: UIView = {
        let dimmingView = UIView()
        dimmingView.backgroundColor = UIMetrics.DimmingView.backgroundColor
        return dimmingView
    }()

    override var shouldRemovePresentersView: Bool {
        return false
    }

    /**
     Flag indicating whether presentation controller should use fullscreen presentation when in
     compact width environment
     */
    var useFullScreenPresentationInCompactWidth = false

    /**
     Returns `true` if presentation controller is in fullscreen presentation.
     */
    var isInFullScreenPresentation: Bool {
        return useFullScreenPresentationInCompactWidth &&
            traitCollection.horizontalSizeClass == .compact
    }

    override init(presentedViewController: UIViewController, presenting presentingViewController: UIViewController?) {
        super.init(presentedViewController: presentedViewController, presenting: presentingViewController)
        addKeyboardResponder()
    }

    override var frameOfPresentedViewInContainerView: CGRect {
        guard let containerView else {
            return super.frameOfPresentedViewInContainerView
        }

        if isInFullScreenPresentation {
            return containerView.bounds
        }

        let preferredContentSize = presentedViewController.preferredContentSize

        assert(preferredContentSize.width > 0 && preferredContentSize.height > 0)

        return CGRect(
            origin: CGPoint(
                x: containerView.bounds.midX - preferredContentSize.width * 0.5,
                y: containerView.bounds.midY - preferredContentSize.height * 0.5
            ),
            size: preferredContentSize
        )
    }

    override func containerViewWillLayoutSubviews() {
        dimmingView.frame = containerView?.bounds ?? .zero
        presentedView?.frame = frameOfPresentedViewInContainerView
    }

    override func presentationTransitionWillBegin() {
        dimmingView.alpha = 0
        containerView?.addSubview(dimmingView)

        presentedView?.layer.cornerRadius = UIMetrics.DimmingView.cornerRadius
        presentedView?.clipsToBounds = true

        let revealDimmingView = {
            self.dimmingView.alpha = UIMetrics.DimmingView.opacity
        }

        if let transitionCoordinator = presentingViewController.transitionCoordinator {
            transitionCoordinator.animate { context in
                revealDimmingView()
            }
        } else {
            revealDimmingView()
        }

        postFullscreenPresentationWillChangeIfNeeded()
    }

    override func presentationTransitionDidEnd(_ completed: Bool) {
        if !completed {
            dimmingView.removeFromSuperview()
        }
    }

    override func dismissalTransitionWillBegin() {
        let fadeDimmingView = {
            self.dimmingView.alpha = 0
        }

        if let transitionCoordinator = presentingViewController.transitionCoordinator {
            transitionCoordinator.animate { context in
                fadeDimmingView()
            }
        } else {
            fadeDimmingView()
        }
    }

    override func dismissalTransitionDidEnd(_ completed: Bool) {
        if completed {
            dimmingView.removeFromSuperview()
        }
    }

    override func traitCollectionDidChange(_ previousTraitCollection: UITraitCollection?) {
        super.traitCollectionDidChange(previousTraitCollection)

        postFullscreenPresentationWillChangeIfNeeded()
    }

    private func postFullscreenPresentationWillChangeIfNeeded() {
        let currentIsInFullScreen = isInFullScreenPresentation

        guard lastKnownIsInFullScreen != currentIsInFullScreen else { return }

        lastKnownIsInFullScreen = currentIsInFullScreen

        NotificationCenter.default.post(
            name: Self.willChangeFullScreenPresentation,
            object: presentedViewController,
            userInfo: [Self.isFullScreenUserInfoKey: NSNumber(booleanLiteral: currentIsInFullScreen)]
        )
    }

    private func addKeyboardResponder() {
        keyboardResponder = .init(targetView: presentedView!, handler: { [weak self] view, adjustment in
            guard let self, let containerView else { return }
            let frame = view.frame
            let margin = adjustment > 0
                ? UIMetrics.sectionSpacing

                : 0
            view.frame = .init(
                origin: CGPoint(
                    x: frame.origin.x,
                    y: containerView.bounds.midY - presentedViewController.preferredContentSize
                        .height * 0.5 - adjustment - margin
                ),
                size: frame.size
            )
            view.layoutIfNeeded()
        })
    }
}

class FormSheetTransitioningDelegate: NSObject, UIViewControllerTransitioningDelegate {
    func animationController(
        forPresented presented: UIViewController,
        presenting: UIViewController,
        source: UIViewController
    ) -> UIViewControllerAnimatedTransitioning? {
        return FormSheetPresentationAnimator()
    }

    func animationController(forDismissed dismissed: UIViewController)
        -> UIViewControllerAnimatedTransitioning?
    {
        return FormSheetPresentationAnimator()
    }

    func presentationController(
        forPresented presented: UIViewController,
        presenting: UIViewController?,
        source: UIViewController
    ) -> UIPresentationController? {
        return FormSheetPresentationController(
            presentedViewController: presented,
            presenting: source
        )
    }
}

class FormSheetPresentationAnimator: NSObject, UIViewControllerAnimatedTransitioning {
    func transitionDuration(using transitionContext: UIViewControllerContextTransitioning?)
        -> TimeInterval
    {
        return (transitionContext?.isAnimated ?? true) ? UIMetrics.Transition.duration : 0
    }

    func animateTransition(using transitionContext: UIViewControllerContextTransitioning) {
        let destination = transitionContext.viewController(forKey: .to)

        if destination?.isBeingPresented ?? false {
            animatePresentation(transitionContext)
        } else {
            animateDismissal(transitionContext)
        }
    }

    private func animatePresentation(_ transitionContext: UIViewControllerContextTransitioning) {
        let duration = transitionDuration(using: transitionContext)
        let containerView = transitionContext.containerView
        let destinationView = transitionContext.view(forKey: .to)!
        let destinationController = transitionContext.viewController(forKey: .to)!

        containerView.addSubview(destinationView)

        var initialFrame = transitionContext.finalFrame(for: destinationController)
        initialFrame.origin.y = containerView.bounds.maxY
        destinationView.frame = initialFrame

        if transitionContext.isAnimated {
            UIView.animate(
                withDuration: duration,
                delay: UIMetrics.Transition.delay,
                options: UIMetrics.Transition.animationOptions,
                animations: {
                    destinationView.frame = transitionContext.finalFrame(for: destinationController)
                },
                completion: { _ in
                    transitionContext.completeTransition(true)
                }
            )
        } else {
            destinationView.frame = transitionContext.finalFrame(for: destinationController)
        }
    }

    private func animateDismissal(_ transitionContext: UIViewControllerContextTransitioning) {
        let duration = transitionDuration(using: transitionContext)
        let containerView = transitionContext.containerView
        let sourceView = transitionContext.view(forKey: .from)!
        let sourceController = transitionContext.viewController(forKey: .from)!

        var initialFrame = transitionContext.finalFrame(for: sourceController)
        initialFrame.origin.y = containerView.bounds.maxY

        if transitionContext.isAnimated {
            UIView.animate(
                withDuration: duration,
                delay: UIMetrics.Transition.delay,
                options: UIMetrics.Transition.animationOptions,
                animations: {
                    sourceView.frame = initialFrame
                },
                completion: { _ in
                    transitionContext.completeTransition(true)
                }
            )
        } else {
            sourceView.frame = initialFrame
            transitionContext.completeTransition(true)
        }
    }
}
