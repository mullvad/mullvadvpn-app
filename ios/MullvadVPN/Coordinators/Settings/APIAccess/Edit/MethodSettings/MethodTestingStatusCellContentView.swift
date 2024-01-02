//
//  MethodTestingStatusContentCell.swift
//  MullvadVPN
//
//  Created by pronebird on 16/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

/// Content view presenting the access method testing progress.
class MethodTestingStatusCellContentView: UIView, UIContentView {
    private let progressView = SpinnerActivityIndicatorView(style: .custom)
    private let progressContainer = UIView()
    private let containerView = UIView()

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

    var configuration: UIContentConfiguration {
        get {
            actualConfiguration
        }
        set {
            guard let newConfiguration = newValue as? MethodTestingStatusCellContentConfiguration else { return }

            let previousConfiguration = actualConfiguration
            actualConfiguration = newConfiguration

            configureSubviews(previousConfiguration: previousConfiguration)
        }
    }

    private var actualConfiguration: MethodTestingStatusCellContentConfiguration

    func supports(_ configuration: UIContentConfiguration) -> Bool {
        configuration is MethodTestingStatusCellContentConfiguration
    }

    init(configuration: MethodTestingStatusCellContentConfiguration) {
        actualConfiguration = configuration

        super.init(frame: CGRect(x: 0, y: 0, width: 100, height: 44))

        addSubviews()
        configureSubviews()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func addSubviews() {
        NSLayoutConstraint.activate([
            progressView.widthAnchor.constraint(equalToConstant: 30),
            progressView.heightAnchor.constraint(equalToConstant: 30),

            progressContainer.widthAnchor.constraint(equalToConstant: 30),
            progressContainer.heightAnchor.constraint(equalToConstant: 20),

            statusIndicator.widthAnchor.constraint(equalToConstant: 20),
            statusIndicator.heightAnchor.constraint(equalToConstant: 20).withPriority(.defaultHigh),
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
            verticalStackView.pinEdgesToSuperviewMargins()
        }
    }

    private func configureSubviews(previousConfiguration: MethodTestingStatusCellContentConfiguration? = nil) {
        configureLayoutMargins()

        textLabel.text = actualConfiguration.status.text
        detailLabel.text = actualConfiguration.detailText
        statusIndicator.backgroundColor = actualConfiguration.status.statusColor

        // Remove the first view in the horizontal stack which is either a status indicator or progress.
        horizontalStackView.arrangedSubviews.first.map { view in
            horizontalStackView.removeArrangedSubview(view)
            view.removeFromSuperview()
        }

        // Reconfigure the horizontal stack by adding the status indicator or progress first.
        switch actualConfiguration.status {
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

    private func configureLayoutMargins() {
        directionalLayoutMargins = actualConfiguration.directionalLayoutMargins
    }
}
