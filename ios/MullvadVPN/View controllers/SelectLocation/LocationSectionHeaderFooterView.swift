import Foundation
import SwiftUI
import UIKit

class LocationSectionHeaderFooterView: UITableViewHeaderFooterView {
    static let reuseIdentifier = "LocationSectionHeaderFooterView"

    private let label = UILabel()
    private let button = UIButton(type: .system)

    override init(reuseIdentifier: String?) {
        super.init(reuseIdentifier: reuseIdentifier)

        // Configure button
        button.setImage(UIImage(systemName: "ellipsis"), for: .normal)
        button.tintColor = UIColor(white: 1, alpha: 0.6)

        contentView.addConstrainedSubviews([label, button]) {
            label.pinEdgesToSuperviewMargins(.all().excluding(.trailing))
            button.pinEdgesToSuperviewMargins(.all().excluding(.leading))
            button.leadingAnchor.constraint(greaterThanOrEqualTo: label.trailingAnchor, constant: 8)
        }
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    func configure(configuration: Configuration) {
        var contentConfig = UIListContentConfiguration.groupedHeader()
        contentConfig.text = configuration.name
        contentConfig.textProperties.alignment = configuration.style.textAlignment
        contentConfig.textProperties.color = configuration.style.textColor
        contentConfig.textProperties.font = configuration.style.font
        contentConfig.textProperties.adjustsFontForContentSizeCategory = true

        contentView.backgroundColor = configuration.style.backgroundColor
        directionalLayoutMargins = configuration.directionalEdgeInsets

        // Apply the font and color directly to the label:
        label.text = configuration.name
        label.font = contentConfig.textProperties.font
        label.textColor = contentConfig.textProperties.color
        label.adjustsFontForContentSizeCategory = true
        label.numberOfLines = 0
        label.lineBreakMode = .byWordWrapping
        label.setContentCompressionResistancePriority(.required, for: .horizontal)

        if let buttonAction = configuration.primaryAction {
            button.isHidden = false
            button.removeTarget(nil, action: nil, for: .allEvents)
            button.addAction(buttonAction, for: .touchUpInside)
        } else {
            button.isHidden = true
        }
    }
}

extension LocationSectionHeaderFooterView {
    struct Style: Equatable, @unchecked Sendable {
        let font: UIFont
        let textColor: UIColor
        let textAlignment: UIListContentConfiguration.TextProperties.TextAlignment
        let backgroundColor: UIColor

        static let header = Style(
            font: .mullvadSmallSemiBold,
            textColor: .primaryTextColor,
            textAlignment: .natural,
            backgroundColor: .primaryColor
        )

        static let footer = Style(
            font: .mullvadTiny,
            textColor: .secondaryTextColor,
            textAlignment: .center,
            backgroundColor: .clear
        )
    }

    struct Configuration {
        let name: String
        let style: Style
        var directionalEdgeInsets = NSDirectionalEdgeInsets(top: 11, leading: 16, bottom: 11, trailing: 8)
        var primaryAction: UIAction?
    }
}
