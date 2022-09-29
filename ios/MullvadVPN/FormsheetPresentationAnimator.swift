//
//  FormsheetPresentationAnimator.swift
//  MullvadVPN
//
//  Created by Andreas Lif on 2022-08-30.
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
        guard let destinationController = transitionContext.viewController(forKey: .to)
        else { return }

        if destinationController.isBeingPresented {
            animatePresentation(transitionContext)
        } else {
            animateDismissal(transitionContext)
        }
    }

    private func animatePresentation(_ transitionContext: UIViewControllerContextTransitioning) {
        guard let destinationView = transitionContext.view(forKey: .to),
              let destinationController = transitionContext.viewController(forKey: .to)
        else { return }

        let containerView = transitionContext.containerView
        containerView.addSubview(destinationView)

        destinationView.frame = initialFrame(
            destinationController,
            transitionContext: transitionContext
        )

        let finalFrame = transitionContext.finalFrame(for: destinationController)

        UIView.animate(
            withDuration: transitionDuration(using: transitionContext),
            delay: 0,
            options: .curveEaseOut,
            animations: {
                destinationView.frame = finalFrame
            },
            completion: { _ in
                transitionContext.completeTransition(true)
            }
        )
    }

    private func animateDismissal(_ transitionContext: UIViewControllerContextTransitioning) {
        guard let sourceView = transitionContext.view(forKey: .from),
              let sourceController = transitionContext.viewController(forKey: .from)
        else { return }

        let finalFrame = initialFrame(sourceController, transitionContext: transitionContext)

        UIView.animate(
            withDuration: transitionDuration(using: transitionContext),
            delay: 0,
            options: .curveLinear,
            animations: {
                sourceView.frame = finalFrame
            },
            completion: { _ in
                transitionContext.completeTransition(true)
            }
        )
    }

    private func initialFrame(
        _ destinationController: UIViewController,
        transitionContext: UIViewControllerContextTransitioning
    ) -> CGRect {
        let presentationController = destinationController
            .presentationController as? FormsheetPresentationController

        if let frame = presentationController?.initialFrameOfPresentedViewInContainerView {
            return frame
        } else {
            var frame = transitionContext.finalFrame(for: destinationController)
            frame.origin.y = transitionContext.containerView.bounds.maxY
            return frame
        }
    }
}
