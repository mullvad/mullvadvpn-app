//
//  SelectLocationCell.swift
//  MullvadVPN
//
//  Created by pronebird on 02/05/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import UIKit

class SelectLocationCell: BasicTableViewCell {
    typealias CollapseHandler = (SelectLocationCell) -> Void

    @IBOutlet var locationLabel: UILabel!
    @IBOutlet var statusIndicator: RelayStatusIndicatorView!
    @IBOutlet var tickImageView: UIImageView!
    @IBOutlet var collapseButton: UIButton!

    private let chevronDown = UIImage(imageLiteralResourceName: "IconChevronDown")
    private let chevronUp = UIImage(imageLiteralResourceName: "IconChevronUp")

    var isDisabled = false {
        didSet {
            updateDisabled()
            updateBackgroundColor()
        }
    }

    var isExpanded = false {
        didSet {
            updateCollapseImage()
        }
    }

    var showsCollapseControl = false {
        didSet {
            collapseButton.isHidden = !showsCollapseControl
        }
    }

    var didCollapseHandler: CollapseHandler?

    private let preferredMargins = UIEdgeInsets(top: 16, left: 28, bottom: 16, right: 12)

    override var indentationLevel: Int {
        didSet {
            updateBackgroundColor()
        }
    }

    override func awakeFromNib() {
        super.awakeFromNib()

        indentationWidth = 16
        statusIndicator.tintColor = .white

        collapseButton.addTarget(self, action: #selector(handleCollapseButton(_ :)), for: .touchUpInside)

        updateCollapseImage()
        updateDisabled()
        updateBackgroundColor()

        contentView.layoutMargins = preferredMargins
    }

    override func layoutSubviews() {
        super.layoutSubviews()

        let indentPoints = CGFloat(indentationLevel) * indentationWidth

        contentView.frame = CGRect(
            x: indentPoints,
            y: contentView.frame.origin.y,
            width: contentView.frame.size.width - indentPoints,
            height: contentView.frame.size.height
        )
    }

    override func setSelected(_ selected: Bool, animated: Bool) {
        super.setSelected(selected, animated: animated)

        updateTickImage()
    }

    private func updateTickImage() {
        statusIndicator.isHidden = isSelected
        tickImageView?.isHidden = !isSelected
    }

    private func updateDisabled() {
        contentView.alpha = isDisabled ? 0.5 : 1
    }

    private func updateBackgroundColor() {
        backgroundView?.backgroundColor = backgroundColorForIdentationLevel()
        selectedBackgroundView?.backgroundColor = selectedBackgroundColorForIndentationLevel()
    }

    private func backgroundColorForIdentationLevel() -> UIColor {
        if isDisabled {
            switch indentationLevel {
            case 1:
                return UIColor.SubCell.disabledBackgroundColor
            case 2:
                return UIColor.SubSubCell.disabledBackgroundColor
            default:
                return UIColor.Cell.disabledBackgroundColor
            }
        } else {
            switch indentationLevel {
            case 1:
                return UIColor.SubCell.backgroundColor
            case 2:
                return UIColor.SubSubCell.backgroundColor
            default:
                return UIColor.Cell.backgroundColor
            }
        }
    }

    private func selectedBackgroundColorForIndentationLevel() -> UIColor {
        if isDisabled {
            return UIColor.Cell.disabledSelectedBackgroundColor
        } else {
            return UIColor.Cell.selectedBackgroundColor
        }
    }

    @objc private func handleCollapseButton(_ sender: UIControl) {
        didCollapseHandler?(self)
    }

    private func updateCollapseImage() {
        let image = isExpanded ? chevronUp : chevronDown

        collapseButton.setImage(image, for: .normal)
    }
}
