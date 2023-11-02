//
//  AddAccessMethodActionSheetPresentation.swift
//  MullvadVPN
//
//  Created by pronebird on 16/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

/// Class responsible for presentation of access method sheet within the hosting view.
class AccessMethodActionSheetPresentation {
    /// The view managed by the sheet presentation.
    private let presentationView = AccessMethodActionSheetPresentationView(frame: CGRect(
        x: 0,
        y: 0,
        width: 320,
        height: 240
    ))

    /// Indicates whether the sheet is being presented.
    private(set) var isPresenting = false

    /// Layout frame of a sheet view.
    var sheetLayoutFrame: CGRect {
        presentationView.sheetLayoutFrame
    }

    /// Delegate.
    weak var delegate: AccessMethodActionSheetPresentationDelegate?

    /// Presentation configuration.
    var configuration: AccessMethodActionSheetPresentationConfiguration {
        get {
            presentationView.configuration
        }
        set {
            presentationView.configuration = newValue
        }
    }

    init() {
        presentationView.sheetDelegate = self
        presentationView.alpha = 0
    }

    /// Present the sheet within the hosting view.
    ///
    /// - Parameters:
    ///   - parent: the hosting view.
    ///   - animated: whether to animate the transition.
    func show(in parent: UIView, animated: Bool = true) {
        guard !isPresenting || presentationView.superview != parent else { return }

        isPresenting = true
        embed(into: parent)

        UIViewPropertyAnimator.runningPropertyAnimator(
            withDuration: animated ? UIMetrics.AccessMethodActionSheetTransition.duration.timeInterval : 0,
            delay: 0,
            options: UIMetrics.AccessMethodActionSheetTransition.animationOptions
        ) {
            self.presentationView.alpha = 1
        }
    }

    /// Hide the sheet from the hosting view.
    ///
    /// The sheet is removed from the hosting view after animation.
    ///
    /// - Parameter animated: whether to animate the transition.
    func hide(animated: Bool = true) {
        guard isPresenting else { return }

        isPresenting = false

        UIViewPropertyAnimator.runningPropertyAnimator(
            withDuration: animated ? UIMetrics.AccessMethodActionSheetTransition.duration.timeInterval : 0,
            delay: 0,
            options: UIMetrics.AccessMethodActionSheetTransition.animationOptions
        ) {
            self.presentationView.alpha = 0
        } completion: { position in
            guard position == .end else { return }

            self.presentationView.removeFromSuperview()
        }
    }

    /// Embed the container into the sheet container view into the hosting view.
    ///
    /// - Parameter parent: the hosting view.
    private func embed(into parent: UIView) {
        guard presentationView.superview != parent else { return }

        presentationView.removeFromSuperview()
        parent.addConstrainedSubviews([presentationView]) {
            presentationView.pinEdgesToSuperview()
        }
    }
}

extension AccessMethodActionSheetPresentation: AccessMethodActionSheetDelegate {
    func sheetDidAdd(_ sheet: AccessMethodActionSheetContainerView) {
        delegate?.sheetDidAdd(sheetPresentation: self)
    }

    func sheetDidCancel(_ sheet: AccessMethodActionSheetContainerView) {
        delegate?.sheetDidCancel(sheetPresentation: self)
    }
}
