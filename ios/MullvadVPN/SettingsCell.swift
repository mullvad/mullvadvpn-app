//
//  SettingsCell.swift
//  MullvadVPN
//
//  Created by pronebird on 22/05/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import UIKit

enum SettingsDisclosureType {
    case none
    case chevron
    case externalLink
    case tick

    var image: UIImage? {
        switch self {
        case .none:
            return nil
        case .chevron:
            return UIImage(named: "IconChevron")
        case .externalLink:
            return UIImage(named: "IconExtlink")
        case .tick:
            return .iconTickSmall
        }
    }
}

class SettingsCell: UITableViewCell {
    let titleLabel = UILabel()
    let detailTitleLabel = UILabel()
    let disclosureImageView = UIImageView(image: nil)

    var disclosureType: SettingsDisclosureType = .none {
        didSet {
            accessoryType = .none

            let image = disclosureType.image?.backport_withTintColor(
                UIColor.Cell.disclosureIndicatorColor,
                renderingMode: .alwaysOriginal
            )

            if let image = image {
                disclosureImageView.image = image
                disclosureImageView.sizeToFit()
                accessoryView = disclosureImageView
            } else {
                accessoryView = nil
            }
        }
    }

    override init(style: UITableViewCell.CellStyle, reuseIdentifier: String?) {
        super.init(style: style, reuseIdentifier: reuseIdentifier)

        backgroundView = UIView()
        backgroundView?.backgroundColor = UIColor.Cell.backgroundColor

        selectedBackgroundView = UIView()
        selectedBackgroundView?.backgroundColor = UIColor.Cell.selectedAltBackgroundColor

        separatorInset = .zero
        backgroundColor = .clear
        contentView.backgroundColor = .clear

        titleLabel.translatesAutoresizingMaskIntoConstraints = false
        titleLabel.font = UIFont.systemFont(ofSize: 17)
        titleLabel.textColor = UIColor.Cell.titleTextColor

        detailTitleLabel.translatesAutoresizingMaskIntoConstraints = false
        detailTitleLabel.font = UIFont.systemFont(ofSize: 13)
        detailTitleLabel.textColor = UIColor.Cell.detailTextColor

        titleLabel.setContentHuggingPriority(.defaultHigh, for: .horizontal)
        detailTitleLabel.setContentHuggingPriority(.defaultLow, for: .horizontal)

        titleLabel.setContentCompressionResistancePriority(.defaultHigh, for: .horizontal)
        detailTitleLabel.setContentCompressionResistancePriority(.defaultLow, for: .horizontal)

        contentView.addSubview(titleLabel)
        contentView.addSubview(detailTitleLabel)

        setLayoutMargins()

        NSLayoutConstraint.activate([
            titleLabel.leadingAnchor
                .constraint(equalTo: contentView.layoutMarginsGuide.leadingAnchor),
            titleLabel.topAnchor.constraint(equalTo: contentView.layoutMarginsGuide.topAnchor),
            titleLabel.bottomAnchor
                .constraint(equalTo: contentView.layoutMarginsGuide.bottomAnchor),

            detailTitleLabel.leadingAnchor.constraint(
                greaterThanOrEqualToSystemSpacingAfter: titleLabel.trailingAnchor,
                multiplier: 1
            ),

            detailTitleLabel.trailingAnchor
                .constraint(equalTo: contentView.layoutMarginsGuide.trailingAnchor),
            detailTitleLabel.topAnchor
                .constraint(equalTo: contentView.layoutMarginsGuide.topAnchor),
            detailTitleLabel.bottomAnchor
                .constraint(equalTo: contentView.layoutMarginsGuide.bottomAnchor),
        ])
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func prepareForReuse() {
        super.prepareForReuse()

        setLayoutMargins()
    }

    override func layoutSubviews() {
        super.layoutSubviews()

        if #available(iOS 13, *) {
            // no-op
        } else {
            layoutSubviews_iOS12()
        }
    }

    private func setLayoutMargins() {
        // Set layout margins for standard acceessories added into the cell (reorder control, etc..)
        layoutMargins = UIMetrics.settingsCellLayoutMargins

        // Set layout margins for cell content
        contentView.layoutMargins = UIMetrics.settingsCellLayoutMargins
    }

    /// On iOS 12, standard edit and reorder controls do not respect layout margins.
    /// This method does layout adjustments to fix that.
    private func layoutSubviews_iOS12() {
        guard isEditing || showsReorderControl else { return }

        var leftOffset: CGFloat = 0
        var rightOffset: CGFloat = 0

        for subview in subviews {
            // Detect the edit control and move it, so that the nested image view is aligned along the left edge of the
            // layout margins.
            if subview.description.starts(with: "<UITableViewCellEditControl"),
               let imageView = subview.subviews.first
            {
                let imageOffset = imageView.frame.minX
                var pos = subview.frame.origin
                pos.x = layoutMargins.left - imageOffset
                subview.frame.origin = pos
                leftOffset = pos.x
            }

            // Detect the reorder control and move it, so that its right edge is aligned along the right edge of the
            // layout margins.
            if subview.description.starts(with: "<UITableViewCellReorderControl") {
                var pos = subview.frame.origin
                pos.x -= layoutMargins.right
                subview.frame.origin = pos
                rightOffset = layoutMargins.right
            }
        }

        // Adjust the content view to account for the adjustments to the edit and reorder controls.
        let contentInset = UIEdgeInsets(top: 0, left: leftOffset, bottom: 0, right: rightOffset)
        contentView.frame = contentView.frame.inset(by: contentInset)
    }
}

private extension UIImage {
    static let iconTickSmall: UIImage? = {
        guard let image = UIImage(named: "IconTick") else { return nil }
        let size = CGSize(width: 16, height: 16)
        return UIGraphicsImageRenderer(size: size).image { context in
            let rect = CGRect(origin: .zero, size: size)
            image.draw(in: rect)
        }
    }()
}
