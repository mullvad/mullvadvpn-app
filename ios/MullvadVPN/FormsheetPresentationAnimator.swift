//
//  FormsheetPresentationAnimator.swift
//  MullvadVPN
//
//  Created by pronebird on 2022-12-16.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import UIKit

final class FormsheetPresentationAnimator: NSObject, UIViewControllerAnimatedTransitioning {
    func transitionDuration(using transitionContext: UIViewControllerContextTransitioning?)
        -> TimeInterval
    {
        return (transitionContext?.isAnimated ?? true) ? 0.25 : 0
    }

    func animateTransition(using transitionContext: UIViewControllerContextTransitioning) {
        let destination = transitionContext.viewController(forKey: .to)

        if destination?.isBeingPresented ?? false {
            animatePresentation(transitionContext)
        } else {
            animateDismissal(transitionContext)
        }
    }

    static func initialFrame(in containerBounds: CGRect, preferredContentSize: CGSize) -> CGRect {
        let presentedViewSize = presentedViewSize(
            containerSize: containerBounds.size,
            preferredContentSize: preferredContentSize
        )

        return CGRect(
            origin: CGPoint(
                x: containerBounds.midX - presentedViewSize.width * 0.5,
                y: containerBounds.maxY
            ),
            size: presentedViewSize
        )
    }

    static func targetFrame(in containerBounds: CGRect, preferredContentSize: CGSize) -> CGRect {
        let presentedViewSize = presentedViewSize(
            containerSize: containerBounds.size,
            preferredContentSize: preferredContentSize
        )

        return CGRect(
            origin: CGPoint(
                x: containerBounds.midX - presentedViewSize.width * 0.5,
                y: containerBounds.midY - presentedViewSize.height * 0.5
            ),
            size: presentedViewSize
        )
    }

    static func presentedViewSize(containerSize: CGSize, preferredContentSize: CGSize) -> CGSize {
        return CGSize(
            width: min(containerSize.width, preferredContentSize.width),
            height: min(containerSize.height, preferredContentSize.height)
        )
    }

    private func animatePresentation(_ transitionContext: UIViewControllerContextTransitioning) {
        let duration = transitionDuration(using: transitionContext)
        let containerView = transitionContext.containerView
        let destinationView = transitionContext.view(forKey: .to) ?? UIView()
        let destinationController = transitionContext
            .viewController(forKey: .to) ?? UIViewController()
        let preferredContentSize = destinationController.preferredContentSize

        containerView.addSubview(destinationView)
        destinationView.frame = Self.initialFrame(
            in: containerView.bounds,
            preferredContentSize: preferredContentSize
        )

        UIView.animate(
            withDuration: duration,
            delay: 0,
            options: .curveEaseOut,
            animations: {
                destinationView.frame = Self.targetFrame(
                    in: containerView.bounds,
                    preferredContentSize: preferredContentSize
                )
            },
            completion: { _ in
                transitionContext.completeTransition(true)
            }
        )
    }

    private func animateDismissal(_ transitionContext: UIViewControllerContextTransitioning) {
        let duration = transitionDuration(using: transitionContext)
        let containerView = transitionContext.containerView
        let sourceView = transitionContext.view(forKey: .from) ?? UIView()
        let sourceController = transitionContext.viewController(forKey: .from) ?? UIViewController()
        let preferredContentSize = sourceController.preferredContentSize

        UIView.animate(
            withDuration: duration,
            delay: 0,
            options: .curveEaseIn,
            animations: {
                sourceView.frame = Self.initialFrame(
                    in: containerView.bounds,
                    preferredContentSize: preferredContentSize
                )
            },
            completion: { _ in
                transitionContext.completeTransition(true)
            }
        )
    }
}
