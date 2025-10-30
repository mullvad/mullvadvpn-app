//
//  ReduceAnonimityWarningView.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2025-10-27.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

class ReduceAnonymityWarningView: UIView {
    var viewIsExpanded = false
    var warningTextLabel: UILabel!
    var warningTextContainer: UIStackView!
    var chevronIcon: UIImageView!

    override init(frame: CGRect) {
        super.init(frame: frame)

        let warningIcon = UIImageView(image: UIImage(systemName: "exclamationmark.circle"))
        warningIcon.contentMode = .scaleAspectFit
        warningIcon.setContentHuggingPriority(.defaultHigh, for: .horizontal)
        warningIcon.tintColor = UIColor.warningColor

        let warningHeader = UILabel()
        warningHeader.text = NSLocalizedString("This impacts your anonymity", comment: "")
        warningHeader.translatesAutoresizingMaskIntoConstraints = false
        warningHeader.font = .mullvadMiniSemiBold
        warningHeader.numberOfLines = 0
        warningHeader.textColor = .white
        warningHeader.adjustsFontForContentSizeCategory = true

        let normalTextAttributes: [NSAttributedString.Key: Any] = [
            .font: UIFont.mullvadMini,
            .foregroundColor: UIColor.white,
        ]

        let linkAttributes: [NSAttributedString.Key: Any] = [
            .font: UIFont.mullvadMini,
            .foregroundColor: UIColor.ContentHeading.linkColor,
            .underlineStyle: NSUnderlineStyle.single.rawValue,
            .underlineColor: UIColor.ContentHeading.linkColor,
        ]

        let warningTextLabel = UILabel()
        self.warningTextLabel = warningTextLabel
        let attributedText = NSMutableAttributedString()
        let warningText = NSLocalizedString(
            "By attaching your account token it links this report to your account, which helps us resolve your issue quicker. All reports are automatically deleted after a period of time. For details, please see our ",
            comment: "")
        let linkText = NSLocalizedString("privacy policy", comment: "")

        let paragraphStyle = NSMutableParagraphStyle()
        paragraphStyle.lineBreakMode = .byWordWrapping

        attributedText.append(NSAttributedString(string: warningText, attributes: normalTextAttributes))
        attributedText.append(NSAttributedString(string: linkText, attributes: linkAttributes))
        attributedText.addAttribute(
            .paragraphStyle,
            value: paragraphStyle,
            range: NSRange(location: 0, length: attributedText.length)
        )
        attributedText.addAttribute(
            .foregroundColor, value: UIColor.white,
            range: NSRange(location: 0, length: attributedText.length))

        warningTextLabel.attributedText = attributedText
        warningTextLabel.translatesAutoresizingMaskIntoConstraints = false
        warningTextLabel.numberOfLines = 0
        warningTextLabel.adjustsFontForContentSizeCategory = true
        warningTextLabel.isHidden = true
        warningTextLabel.isUserInteractionEnabled = true

        warningTextLabel.addGestureRecognizer(
            UITapGestureRecognizer(target: self, action: #selector(openPrivacyPolicy)))

        let chevronIcon = UIImageView(image: .CellDecoration.chevronDown)
        self.chevronIcon = chevronIcon
        chevronIcon.contentMode = .scaleAspectFill
        chevronIcon.adjustsImageSizeForAccessibilityContentSizeCategory = true
        chevronIcon.setContentHuggingPriority(.defaultHigh, for: .horizontal)
        chevronIcon.tintColor = .white

        let horizontalStackView = UIStackView()
        horizontalStackView.axis = .horizontal
        horizontalStackView.translatesAutoresizingMaskIntoConstraints = false
        horizontalStackView.spacing = 8

        horizontalStackView.addConstrainedSubviews([warningIcon, warningHeader, chevronIcon]) {
            warningIcon.pinEdgesToSuperviewMargins(PinnableEdges([.leading(4), .top(10), .bottom(0)]))
            chevronIcon.pinEdgesToSuperview(PinnableEdges([.trailing(10), .top(10)]))
            warningIcon.centerYAnchor.constraint(equalTo: chevronIcon.centerYAnchor)

            warningHeader.leadingAnchor.constraint(equalTo: warningIcon.trailingAnchor, constant: 8)
            warningHeader.pinEdgesToSuperview(PinnableEdges([.top(12), .bottom(2)]))
            warningHeader.trailingAnchor.constraint(equalTo: chevronIcon.leadingAnchor, constant: 8)

        }

        let warningTextContainer = UIStackView(arrangedSubviews: [warningTextLabel])
        self.warningTextContainer = warningTextContainer
        warningTextContainer.axis = .horizontal
        warningTextContainer.distribution = .equalSpacing
        warningTextContainer.translatesAutoresizingMaskIntoConstraints = false
        warningTextContainer.spacing = 8

        let verticalStackView = UIStackView()
        verticalStackView.axis = .vertical
        verticalStackView.spacing = 8
        verticalStackView.translatesAutoresizingMaskIntoConstraints = false
        verticalStackView.layer.backgroundColor = CGColor(red: 0.06, green: 0.09, blue: 0.14, alpha: 0.4)
        verticalStackView.layer.cornerRadius = 4

        verticalStackView.addConstrainedSubviews([horizontalStackView, warningTextContainer]) {
            horizontalStackView.pinEdgesToSuperviewMargins(PinnableEdges([.leading(0), .top(0), .trailing(0)]))
            warningTextContainer.topAnchor.constraint(equalTo: horizontalStackView.bottomAnchor, constant: 10)
            warningTextContainer.pinEdgesToSuperviewMargins(PinnableEdges([.leading(4), .trailing(10), .bottom(10)]))
        }

        verticalStackView.addGestureRecognizer(
            UITapGestureRecognizer(target: self, action: #selector(expandAnonymityWarning)))

        addConstrainedSubviews([verticalStackView]) {
            verticalStackView.pinEdgesToSuperviewMargins()
        }
    }

    @objc func expandAnonymityWarning() {
        UIView.animate(withDuration: 0.2) { [weak self] in
            guard let self else { return }
            viewIsExpanded.toggle()
            warningTextLabel.isHidden = !self.viewIsExpanded
            warningTextContainer.isHidden = !self.viewIsExpanded
            chevronIcon.transform = chevronIcon.transform.rotated(by: .pi)
        }
    }

    @objc func openPrivacyPolicy() {
        UIApplication.shared.open(URL(string: ApplicationConfiguration.privacyPolicyLink)!)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}
