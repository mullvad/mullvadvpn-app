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

/// View controller used for presenting a detailed information on some topic using a scrollable stack view.
class AboutViewController: UIViewController {
    private let scrollView = UIScrollView()
    private let contentView = UIStackView()
    private let header: String?
    private let preamble: String?
    private let body: [String]

    init(header: String?, preamble: String?, body: [String]) {
        self.header = header
        self.preamble = preamble
        self.body = body

        super.init(nibName: nil, bundle: nil)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        view.backgroundColor = .secondaryColor
        navigationController?.navigationBar.configureCustomAppeareance()

        setUpContentView()

        scrollView.addConstrainedSubviews([contentView]) {
            contentView.pinEdgesToSuperview()
            contentView.widthAnchor.constraint(equalTo: scrollView.widthAnchor)
        }

        view.addConstrainedSubviews([scrollView]) {
            scrollView.pinEdgesToSuperview()
        }
    }

    private func setUpContentView() {
        contentView.axis = .vertical
        contentView.spacing = 15
        contentView.layoutMargins = UIMetrics.contentInsets
        contentView.isLayoutMarginsRelativeArrangement = true

        if let header {
            let label = UILabel()

            label.text = header
            label.font = .mullvadLarge
            label.adjustsFontForContentSizeCategory = true
            label.textColor = .primaryTextColor
            label.numberOfLines = 0
            label.textAlignment = .center

            contentView.addArrangedSubview(label)
            contentView.setCustomSpacing(32, after: label)
        }

        if let preamble {
            let label = UILabel()

            label.text = preamble
            label.font = .mullvadSmall
            label.adjustsFontForContentSizeCategory = true
            label.textColor = .primaryTextColor
            label.numberOfLines = 0
            label.textAlignment = .center

            contentView.addArrangedSubview(label)
            contentView.setCustomSpacing(24, after: label)
        }

        for text in body {
            let label = UILabel()

            label.text = text
            label.font = .mullvadTiny
            label.adjustsFontForContentSizeCategory = true
            label.textColor = .secondaryTextColor
            label.numberOfLines = 0

            contentView.addArrangedSubview(label)
        }
    }

    static func presentWithNavigationController(_ navigationController: UINavigationController) {
        let header = NSLocalizedString("Server IP override", comment: "")
        let body = [
            NSLocalizedString(
                """
                On some networks, where various types of censorship are being used, our server IP addresses are \
                sometimes blocked.
                """,
                comment: ""
            ),
            NSLocalizedString(
                "To circumvent this you can import a file or a text, provided by our support team, "
                    + "with new IP addresses that override the default addresses of the servers "
                    + "in the Select location view.",
                comment: ""
            ),
            NSLocalizedString(
                "If you are having issues connecting to VPN servers, please contact support.",
                comment: ""
            ),
        ]

        let aboutController = AboutViewController(header: header, preamble: nil, body: body)
        let aboutNavController = UINavigationController(rootViewController: aboutController)

        aboutController.navigationItem.rightBarButtonItem = UIBarButtonItem(
            systemItem: .done,
            primaryAction: UIAction { [weak aboutNavController] _ in
                aboutNavController?.dismiss(animated: true)
            }
        )

        navigationController.present(aboutNavController, animated: true)
    }
}
