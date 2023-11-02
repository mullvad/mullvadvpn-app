//
//  AccessMethodActionSheetPresentationView.swift
//  MullvadVPN
//
//  Created by pronebird on 28/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

/// The sheet presentation view implementing a layout similar to the one used by system action sheet.
class AccessMethodActionSheetPresentationView: UIView {
    /// The dimming background view.
    private let backgroundView: UIView = {
        let backgroundView = UIView()
        backgroundView.backgroundColor = .secondaryColor.withAlphaComponent(0.5)
        return backgroundView
    }()

    /// The blur view displayed behind the sheet.
    private let sheetBlurBackgroundView: UIVisualEffectView = {
        let blurView = UIVisualEffectView(effect: UIBlurEffect(style: .dark))
        blurView.directionalLayoutMargins = .zero
        blurView.contentView.directionalLayoutMargins = .zero
        return blurView
    }()

    /// Sheet container view that contains action buttons and access method testing progress UI.
    private let sheetView = AccessMethodActionSheetContainerView(frame: CGRect(x: 0, y: 0, width: 100, height: 44))

    /// Layout frame of a sheet content view.
    var sheetLayoutFrame: CGRect {
        sheetView.convert(sheetView.bounds, to: self)
    }

    /// Sheet delegate.
    weak var sheetDelegate: AccessMethodActionSheetDelegate? {
        get {
            sheetView.delegate
        }
        set {
            sheetView.delegate = newValue
        }
    }

    /// Presentation configuration.
    var configuration = AccessMethodActionSheetPresentationConfiguration() {
        didSet {
            updateSubviews(previousConfiguration: oldValue, animated: window != nil)
        }
    }

    override init(frame: CGRect) {
        super.init(frame: frame)

        addBackgroundView()
        updateSubviews(animated: false)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func point(inside point: CGPoint, with event: UIEvent?) -> Bool {
        if configuration.dimsBackground {
            super.point(inside: point, with: event)
        } else {
            // Accept touches to the content view only when background view is hidden to enable user interaction with
            // the view beneath.
            sheetView.frame.contains(point)
        }
    }

    private func addBackgroundView() {
        addConstrainedSubviews([backgroundView]) {
            backgroundView.pinEdgesToSuperview()
        }
    }

    private func updateSubviews(
        previousConfiguration: AccessMethodActionSheetPresentationConfiguration? = nil,
        animated: Bool
    ) {
        if previousConfiguration?.blursSheetBackground != configuration.blursSheetBackground {
            updateSheetBackground()
        }

        if previousConfiguration?.dimsBackground != configuration.dimsBackground {
            updateBackgroundView(animated: animated)
        }

        sheetView.configuration = configuration.sheetConfiguration
    }

    private func updateSheetBackground() {
        sheetView.removeFromSuperview()
        sheetBlurBackgroundView.removeFromSuperview()

        // Embed the sheet view into blur view when configured to blur the sheet's background.
        if configuration.blursSheetBackground {
            sheetBlurBackgroundView.contentView.addConstrainedSubviews([sheetView]) {
                sheetView.pinEdgesToSuperviewMargins()
            }
            addConstrainedSubviews([sheetBlurBackgroundView]) {
                sheetBlurBackgroundView.pinEdgesToSuperview(.all().excluding(.top))
            }
        } else {
            addConstrainedSubviews([sheetView]) {
                sheetView.pinEdgesToSuperviewMargins(.all().excluding(.top))
            }
        }
    }

    private func updateBackgroundView(animated: Bool) {
        UIViewPropertyAnimator.runningPropertyAnimator(
            withDuration: animated ? UIMetrics.AccessMethodActionSheetTransition.duration.timeInterval : 0,
            delay: 0,
            options: UIMetrics.AccessMethodActionSheetTransition.animationOptions,
            animations: {
                self.backgroundView.alpha = self.configuration.dimsBackground ? 1 : 0
            }
        )
    }
}
