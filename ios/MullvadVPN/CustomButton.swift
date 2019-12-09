//
//  CustomButton.swift
//  MullvadVPN
//
//  Created by pronebird on 23/05/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import UIKit

@IBDesignable class CustomButton: UIButton {

    override init(frame: CGRect) {
        super.init(frame: frame)

        commonInit()
    }

    required init?(coder aDecoder: NSCoder) {
        super.init(coder: aDecoder)

        commonInit()
    }

    private func commonInit() {
        var contentInsets = contentEdgeInsets

        if contentInsets.top == 0 {
            contentInsets.top = 10
        }

        if contentInsets.bottom == 0 {
            contentInsets.bottom = 10
        }

        if contentInsets.right == 0 {
            contentInsets.right = 10
        }

        if contentInsets.left == 0 {
            contentInsets.left = 10
        }

        contentEdgeInsets = contentInsets
        titleLabel?.font = UIFont.systemFont(ofSize: 17, weight: .semibold)

        setTitleColor(UIColor.white, for: .normal)
        setTitleColor(UIColor.lightGray, for: .highlighted)
    }

    override func imageRect(forContentRect contentRect: CGRect) -> CGRect {
        var imageRect = super.imageRect(forContentRect: contentRect)

        imageRect.origin.x = contentRect.maxX - imageRect.size.width

        return imageRect
    }

    override func titleRect(forContentRect contentRect: CGRect) -> CGRect {
        var titleRect = super.titleRect(forContentRect: contentRect)

        titleRect.origin.x = contentRect.midX - titleRect.width * 0.5

        return titleRect
    }

}
