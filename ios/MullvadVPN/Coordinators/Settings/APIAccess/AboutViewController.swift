//
//  AboutViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 08/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

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
            label.font = .systemFont(ofSize: 28, weight: .bold)
            label.textColor = .white
            label.numberOfLines = 0
            label.textAlignment = .center

            contentView.addArrangedSubview(label)
            contentView.setCustomSpacing(32, after: label)
        }

        if let preamble {
            let label = UILabel()

            label.text = preamble
            label.font = .systemFont(ofSize: 18)
            label.textColor = .white
            label.numberOfLines = 0
            label.textAlignment = .center

            contentView.addArrangedSubview(label)
            contentView.setCustomSpacing(24, after: label)
        }

        for text in body {
            let label = UILabel()

            label.text = text
            label.font = .systemFont(ofSize: 15)
            label.textColor = .white
            label.numberOfLines = 0

            contentView.addArrangedSubview(label)
        }
    }
}
