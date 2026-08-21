// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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
