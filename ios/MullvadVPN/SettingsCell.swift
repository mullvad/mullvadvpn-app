//
//  SettingsCell.swift
//  MullvadVPN
//
//  Created by pronebird on 22/05/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import UIKit

class SettingsCell: BasicTableViewCell {

    let titleLabel = UILabel()
    let detailTitleLabel = UILabel()

    override init(style: UITableViewCell.CellStyle, reuseIdentifier: String?) {
        super.init(style: style, reuseIdentifier: reuseIdentifier)

        backgroundView?.backgroundColor = UIColor.Cell.backgroundColor
        selectedBackgroundView?.backgroundColor = UIColor.Cell.selectedAltBackgroundColor
        separatorInset = .zero

        titleLabel.translatesAutoresizingMaskIntoConstraints = false
        titleLabel.font = UIFont.systemFont(ofSize: 17)
        titleLabel.textColor = .white

        detailTitleLabel.translatesAutoresizingMaskIntoConstraints = false
        detailTitleLabel.font = UIFont.systemFont(ofSize: 13)
        detailTitleLabel.textColor = .white

        titleLabel.setContentHuggingPriority(.defaultHigh, for: .horizontal)
        detailTitleLabel.setContentHuggingPriority(.defaultLow, for: .horizontal)

        titleLabel.setContentCompressionResistancePriority(.defaultHigh, for: .horizontal)
        detailTitleLabel.setContentCompressionResistancePriority(.defaultLow, for: .horizontal)

        contentView.addSubview(titleLabel)
        contentView.addSubview(detailTitleLabel)

        setLayoutMargins()

        NSLayoutConstraint.activate([
            titleLabel.leadingAnchor.constraint(equalTo: contentView.layoutMarginsGuide.leadingAnchor),
            titleLabel.topAnchor.constraint(equalTo: contentView.layoutMarginsGuide.topAnchor),
            titleLabel.bottomAnchor.constraint(equalTo: contentView.layoutMarginsGuide.bottomAnchor),

            detailTitleLabel.leadingAnchor.constraint(greaterThanOrEqualToSystemSpacingAfter: titleLabel.trailingAnchor, multiplier: 1),

            detailTitleLabel.trailingAnchor.constraint(equalTo: contentView.layoutMarginsGuide.trailingAnchor),
            detailTitleLabel.topAnchor.constraint(equalTo: contentView.layoutMarginsGuide.topAnchor),
            detailTitleLabel.bottomAnchor.constraint(equalTo: contentView.layoutMarginsGuide.bottomAnchor),
        ])
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func prepareForReuse() {
        super.prepareForReuse()

        setLayoutMargins()
    }

    override func didAddSubview(_ subview: UIView) {
        super.didAddSubview(subview)

        if let button = subview as? UIButton {
            updateDisclosureIndicatorTintColor(button)
        }
    }

    private func setLayoutMargins() {
        // Set layout margins for standard acceessories added into the cell (reorder control, etc..)
        layoutMargins = UIMetrics.settingsCellLayoutMargins

        // Set layout margins for cell content
        contentView.layoutMargins = UIMetrics.settingsCellLayoutMargins
    }

    /// Standard disclosure views do not provide customization of a tint color.
    /// This method adjusts a disclosure tint color by replacing the button image rendering mode on iOS 12 and by
    /// switching graphics on iOS 13 or newer.
    private func updateDisclosureIndicatorTintColor(_ button: UIButton) {
        guard accessoryType == .disclosureIndicator else { return }

        if #available(iOS 13, *) {
            let configuration = UIImage.SymbolConfiguration(pointSize: 11, weight: .bold)
            let chevron = UIImage(systemName: "chevron.right", withConfiguration: configuration)?
                .withTintColor(.white, renderingMode: .alwaysOriginal)

            button.setImage(chevron, for: .normal)
        } else {
            if let image = button.backgroundImage(for: .normal)?.withRenderingMode(.alwaysTemplate) {
                button.setBackgroundImage(image, for: .normal)
                button.tintColor = .white
            }
        }
    }
}
