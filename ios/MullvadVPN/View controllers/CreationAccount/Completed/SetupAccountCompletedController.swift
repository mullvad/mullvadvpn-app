//
//  SetupAccountCompletedController.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-06-30.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import UIKit

protocol SetupAccountCompletedControllerDelegate: AnyObject, Sendable {
    func didRequestToSeePrivacy(controller: SetupAccountCompletedController)
    func didRequestToStartTheApp(controller: SetupAccountCompletedController)
}

class SetupAccountCompletedController: UIViewController, RootContainment {
    private lazy var contentView: SetupAccountCompletedContentView = {
        let view = SetupAccountCompletedContentView()
        view.delegate = self
        return view
    }()

    var preferredHeaderBarPresentation: HeaderBarPresentation {
        HeaderBarPresentation(style: .default, showsDivider: true)
    }

    var prefersHeaderBarHidden: Bool {
        false
    }

    var prefersDeviceInfoBarHidden: Bool {
        true
    }

    weak var delegate: SetupAccountCompletedControllerDelegate?

    override func viewDidLoad() {
        super.viewDidLoad()
        configureUI()
    }

    private func configureUI() {
        view.addSubview(contentView)
        view.addConstrainedSubviews([contentView]) {
            contentView.pinEdgesToSuperview()
        }
    }
}

extension SetupAccountCompletedController: @preconcurrency SetupAccountCompletedContentViewDelegate {
    func didTapPrivacyButton(view: SetupAccountCompletedContentView, button: AppButton) {
        delegate?.didRequestToSeePrivacy(controller: self)
    }

    func didTapStartingAppButton(view: SetupAccountCompletedContentView, button: AppButton) {
        delegate?.didRequestToStartTheApp(controller: self)
    }
}
