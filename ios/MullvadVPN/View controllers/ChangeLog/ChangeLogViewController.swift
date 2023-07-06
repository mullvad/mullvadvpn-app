//
//  ChangeLogViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 24/03/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

class ChangeLogViewController: UIViewController, RootContainment {
    // MARK: - RootContainment

    var preferredHeaderBarPresentation: HeaderBarPresentation {
        HeaderBarPresentation(style: .default, showsDivider: false)
    }

    var prefersHeaderBarHidden: Bool {
        false
    }

    var prefersNotificationBarHidden: Bool {
        true
    }

    // MARK: - Public

    var onFinish: (() -> Void)?

    func setApplicationVersion(_ string: String) {
        contentView.setApplicationVersion(string)
    }

    func setChangeLogText(_ string: String) {
        contentView.setChangeLogText(string)
    }

    // MARK: - View lifecycle

    private let contentView = ChangeLogContentView()

    override var preferredStatusBarStyle: UIStatusBarStyle {
        .lightContent
    }

    override func loadView() {
        view = contentView

        contentView.didTapButton = { [weak self] in
            self?.onFinish?()
        }
    }
}
