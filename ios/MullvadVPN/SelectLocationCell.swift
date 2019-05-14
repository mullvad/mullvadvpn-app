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

        collapseButton.addTarget(self, action: #selector(handleCollapseButton(_ :)), for: .touchUpInside)

        updateCollapseImage()
    }

    override func layoutMarginsDidChange() {
        super.layoutMarginsDidChange()

        // enforce the preferred layout margins
        if contentView.layoutMargins != preferredMargins {
            contentView.layoutMargins = preferredMargins
        }
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

    private func updateBackgroundColor() {
        backgroundView?.backgroundColor = colorForIdentationLevel()
    }

    private func colorForIdentationLevel() -> UIColor {
        switch indentationLevel {
        case 1:
            return UIColor.subCellBackgroundColor
        case 2:
            return UIColor.subSubCellBackgroundColor
        default:
            return UIColor.cellBackgroundColor
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
