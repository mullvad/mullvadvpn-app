//
//  FormsheetPresentationAnimator.swift
//  MullvadVPN
//
//  Created by Andreas Lif on 2022-08-30.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

class FormsheetPresentationAnimator: NSObject, UIViewControllerAnimatedTransitioning {
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

    static func initialFrame(in containerView: UIView, preferredContentSize: CGSize) -> CGRect {
        return CGRect(
            origin: CGPoint(
                x: containerView.bounds.midX - preferredContentSize.width * 0.5,
                y: containerView.bounds.maxY
            ),
            size: preferredContentSize
        )
    }

    static func targetFrame(in containerView: UIView, preferredContentSize: CGSize) -> CGRect {
        return CGRect(
            origin: CGPoint(
                x: containerView.bounds.midX - preferredContentSize.width * 0.5,
                y: containerView.bounds.midY - preferredContentSize.height * 0.5
            ),
            size: preferredContentSize
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
            in: containerView,
            preferredContentSize: preferredContentSize
        )

        UIView.animate(
            withDuration: duration,
            delay: 0,
            options: .curveEaseOut,
            animations: {
                destinationView.frame = Self.targetFrame(
                    in: containerView,
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
                    in: containerView,
                    preferredContentSize: preferredContentSize
                )
            },
            completion: { _ in
                transitionContext.completeTransition(true)
            }
        )
    }
}
