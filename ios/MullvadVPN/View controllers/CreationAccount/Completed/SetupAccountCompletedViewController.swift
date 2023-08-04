//
//  SetupAccountCompletedViewController.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-06-30.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

protocol SetupAccountCompletedViewControllerDelegate: AnyObject {
    func didRequestToSeePrivacy(controller: SetupAccountCompletedViewController)
    func didRequestToStartTheApp(controller: SetupAccountCompletedViewController)
}

class SetupAccountCompletedViewController: UIViewController, RootContainment {
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

    var prefersNotificationBarHidden: Bool {
        true
    }

    weak var delegate: SetupAccountCompletedViewControllerDelegate?

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

extension SetupAccountCompletedViewController: SetupAccountCompletedContentViewDelegate {
    func didTapPrivacyButton(view: SetupAccountCompletedContentView, button: AppButton) {
        delegate?.didRequestToSeePrivacy(controller: self)
    }

    func didTapStartingAppButton(view: SetupAccountCompletedContentView, button: AppButton) {
        delegate?.didRequestToStartTheApp(controller: self)
    }
}
