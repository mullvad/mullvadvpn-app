//
//  OutOfTimeContentView.swift
//  MullvadVPN
//
//  Created by Andreas Lif on 2022-07-26.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

class OutOfTimeContentView: UIView {
    let statusActivityView: StatusActivityView = {
        let statusActivityView = StatusActivityView(state: .failure)
        statusActivityView.translatesAutoresizingMaskIntoConstraints = false
        return statusActivityView
    }()

    private lazy var titleLabel: UILabel = {
        let label = UILabel()
        label.text = NSLocalizedString("Out of time", comment: "")
        label.font = .mullvadLarge
        label.adjustsFontForContentSizeCategory = true
        label.textColor = .white
        label.numberOfLines = 0
        return label
    }()

    private lazy var bodyLabel: UILabel = {
        let label = UILabel()
        label.textColor = .white
        label.numberOfLines = 0
        label.adjustsFontForContentSizeCategory = true
        return label
    }()

    lazy var disconnectButton: AppButton = {
        let button = AppButton(style: .danger)
        button.translatesAutoresizingMaskIntoConstraints = false
        button.isHidden = true
        let localizedString = NSLocalizedString("Disconnect", comment: "")
        button.setTitle(localizedString, for: .normal)
        return button
    }()

    lazy var purchaseButton: InAppPurchaseButton = {
        let button = InAppPurchaseButton()
        button.translatesAutoresizingMaskIntoConstraints = false
        let localizedString = NSLocalizedString("Add time", comment: "")
        button.setTitle(localizedString, for: .normal)
        return button
    }()

    lazy var restoreButton: AppButton = {
        let button = AppButton(style: .default)
        button.translatesAutoresizingMaskIntoConstraints = false
        button.setTitle(NSLocalizedString("Restore purchases", comment: ""), for: .normal)
        return button
    }()

    private let scrollView = UIScrollView()

    private lazy var topStackView: UIStackView = {
        let stackView = UIStackView(arrangedSubviews: [statusActivityView, titleLabel, bodyLabel])
        stackView.translatesAutoresizingMaskIntoConstraints = false
        stackView.axis = .vertical
        stackView.spacing = UIMetrics.TableView.sectionSpacing
        return stackView
    }()

    private lazy var bottomStackView: UIStackView = {
        let stackView = UIStackView(
            arrangedSubviews: [disconnectButton, purchaseButton, restoreButton]
        )
        stackView.translatesAutoresizingMaskIntoConstraints = false
        stackView.axis = .vertical
        stackView.spacing = UIMetrics.TableView.sectionSpacing
        stackView.backgroundColor = .secondaryColor
        return stackView
    }()

    override init(frame: CGRect) {
        super.init(frame: frame)
        setAccessibilityIdentifier(.outOfTimeView)
        translatesAutoresizingMaskIntoConstraints = false
        backgroundColor = .secondaryColor
        setUpSubviews()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    func enableDisconnectButton(_ enabled: Bool, animated: Bool) {
        disconnectButton.isEnabled = enabled
        UIView.animate(withDuration: animated ? 0.25 : 0) {
            self.disconnectButton.isHidden = !enabled
        }
    }

    func enablePurchaseButton(_ enabled: Bool) {
        purchaseButton.isEnabled = enabled
    }

    // MARK: - Private Functions

    func setUpSubviews() {
        scrollView.addConstrainedSubviews([topStackView]) {
            topStackView.pinEdgesToSuperviewMargins(PinnableEdges([
                .leading(UIMetrics.contentLayoutMargins.leading),
                .trailing(UIMetrics.contentLayoutMargins.trailing),
            ]))
            topStackView.topAnchor.constraint(
                greaterThanOrEqualTo: scrollView.contentLayoutGuide.topAnchor,
                constant: UIMetrics.contentLayoutMargins.top
            )
            topStackView.bottomAnchor.constraint(lessThanOrEqualTo: scrollView.contentLayoutGuide.bottomAnchor)
        }
        addConstrainedSubviews([scrollView]) {
            scrollView.pinEdgesToSuperviewMargins(.all().excluding(.bottom))
        }
        addSubview(bottomStackView)
        configureConstraints()
    }

    func configureConstraints() {
        NSLayoutConstraint.activate([
            bottomStackView.leadingAnchor.constraint(
                equalTo: layoutMarginsGuide.leadingAnchor
            ),
            bottomStackView.trailingAnchor.constraint(
                equalTo: layoutMarginsGuide.trailingAnchor
            ),
            bottomStackView.bottomAnchor.constraint(
                equalTo: layoutMarginsGuide.bottomAnchor
            ),
            scrollView.bottomAnchor.constraint(
                equalTo: bottomStackView.topAnchor
            ),
        ])
    }

    func setBodyLabelText(_ text: String) {
        bodyLabel.attributedText = NSAttributedString(
            markdownString: text,
            options: MarkdownStylingOptions(font: .mullvadSmall)
        )
    }
}
