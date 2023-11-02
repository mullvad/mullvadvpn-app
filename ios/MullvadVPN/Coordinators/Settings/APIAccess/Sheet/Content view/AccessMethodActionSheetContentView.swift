//
//  AccessMethodActionSheetContentView.swift
//  MullvadVPN
//
//  Created by pronebird on 16/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

/// The sheet content view implementing a layout with an activity indicator or status indicator and primary text label, with detail label below.
class AccessMethodActionSheetContentView: UIView {
    var configuration = AccessMethodActionSheetContentConfiguration() {
        didSet {
            updateView()
        }
    }

    private let progressView = SpinnerActivityIndicatorView(style: .custom)
    private let progressContainer = UIView()

    private let statusIndicator: UIView = {
        let view = UIView()
        view.layer.cornerRadius = 10
        view.layer.cornerCurve = .circular
        view.backgroundColor = .successColor
        return view
    }()

    private let textLabel: UILabel = {
        let textLabel = UILabel()
        textLabel.font = UIFont.systemFont(ofSize: 17)
        textLabel.textColor = .primaryTextColor
        return textLabel
    }()

    private let detailLabel: UILabel = {
        let textLabel = UILabel()
        textLabel.font = UIFont.systemFont(ofSize: 14)
        textLabel.textColor = .secondaryTextColor
        textLabel.textAlignment = .center
        return textLabel
    }()

    private let horizontalStackView: UIStackView = {
        let stackView = UIStackView()
        stackView.axis = .horizontal
        stackView.spacing = UIStackView.spacingUseSystem
        stackView.alignment = .center
        return stackView
    }()

    private lazy var verticalStackView: UIStackView = {
        let stackView = UIStackView(arrangedSubviews: [containerView, detailLabel])
        stackView.axis = .vertical
        stackView.alignment = .center
        stackView.spacing = UIStackView.spacingUseSystem
        return stackView
    }()

    private let containerView = UIView()

    init() {
        super.init(frame: CGRect(x: 0, y: 0, width: 100, height: 44))

        setupView()
        updateView()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func setupView() {
        NSLayoutConstraint.activate([
            progressView.widthAnchor.constraint(equalToConstant: 30),
            progressView.heightAnchor.constraint(equalToConstant: 30),

            progressContainer.widthAnchor.constraint(equalToConstant: 30),
            progressContainer.heightAnchor.constraint(equalToConstant: 20),

            statusIndicator.widthAnchor.constraint(equalToConstant: 20),
            statusIndicator.heightAnchor.constraint(equalToConstant: 20),
        ])

        containerView.addConstrainedSubviews([horizontalStackView]) {
            horizontalStackView.pinEdgesToSuperview()
        }

        progressContainer.addConstrainedSubviews([progressView]) {
            progressView.centerYAnchor.constraint(equalTo: progressContainer.centerYAnchor)
            progressView.pinEdgeToSuperview(.leading(0))
            progressView.pinEdgeToSuperview(.trailing(0))
        }

        addConstrainedSubviews([verticalStackView]) {
            verticalStackView.pinEdgesToSuperview()
        }
    }

    private func updateView() {
        textLabel.text = configuration.status.text
        detailLabel.text = configuration.detailText
        statusIndicator.backgroundColor = configuration.status.statusColor

        // Hide detail label when empty to prevent extra margin between subviews in the stack.
        detailLabel.isHidden = configuration.detailText?.isEmpty ?? true

        // Remove the first view in the horizontal stack which is either a status indicator or progress.
        horizontalStackView.arrangedSubviews.first.map { view in
            horizontalStackView.removeArrangedSubview(view)
            view.removeFromSuperview()
        }

        // Reconfigure the horizontal stack by adding the status indicator or progress first.
        switch configuration.status {
        case .reachable, .unreachable:
            horizontalStackView.insertArrangedSubview(statusIndicator, at: 0)

        case .testing:
            horizontalStackView.insertArrangedSubview(progressContainer, at: 0)
            progressView.startAnimating()
        }

        // Text label is always the last one, so only add it into the stack if it's not there yet.
        if textLabel.superview == nil {
            horizontalStackView.addArrangedSubview(textLabel)
        }
    }
}
