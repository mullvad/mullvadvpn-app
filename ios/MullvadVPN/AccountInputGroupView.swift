//
//  AccountInputGroupView.swift
//  MullvadVPN
//
//  Created by pronebird on 22/03/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import UIKit

@IBDesignable class AccountInputGroupView: UIView {

    @IBOutlet var textField: UITextField!

    enum Style {
        case normal, error, authenticating
    }

    var loginState = LoginState.default {
        didSet {
            updateAppearance()
            updateTextFieldEnabled()
        }
    }

    private let borderRadius = CGFloat(8)
    private let borderWidth = CGFloat(2)

    private var borderColor: UIColor {
        switch loginState {
        case .default:
            return textField.isEditing
                ? UIColor.AccountTextField.NormalState.borderColor
                : UIColor.clear

        case .failure:
            return UIColor.AccountTextField.ErrorState.borderColor

        case .authenticating, .success:
            return UIColor.AccountTextField.AuthenticatingState.borderColor
        }
    }

    private var textColor: UIColor {
        switch loginState {
        case .default:
            return UIColor.AccountTextField.NormalState.textColor

        case .failure:
            return UIColor.AccountTextField.ErrorState.textColor

        case .authenticating, .success:
            return UIColor.AccountTextField.AuthenticatingState.textColor
        }
    }

    private var backgroundLayerColor: UIColor {
        switch loginState {
        case .default:
            return UIColor.AccountTextField.NormalState.backgroundColor

        case .failure:
            return UIColor.AccountTextField.ErrorState.backgroundColor

        case .authenticating, .success:
            return UIColor.AccountTextField.AuthenticatingState.backgroundColor
        }
    }

    private let borderLayer = CAShapeLayer()
    private let backgroundLayer = CAShapeLayer()
    private let maskLayer = CALayer()

    // MARK: - View lifecycle

    override func awakeFromNib() {
        super.awakeFromNib()
        setup()
    }

    // MARK: - CALayerDelegate

    override func layoutSublayers(of layer: CALayer) {
        super.layoutSublayers(of: layer)

        guard layer == self.layer else { return }

        // extend the border frame outside of the content area
        let borderFrame = layer.bounds.insetBy(dx: -borderWidth * 0.5, dy: -borderWidth * 0.5)

        // create a bezier path for border
        let borderPath = borderBezierPath(size: borderFrame.size)

        // update the background layer mask
        maskLayer.frame.size = borderFrame.size
        maskLayer.contents = backgroundMaskImage(borderPath: borderPath).cgImage

        backgroundLayer.frame = borderFrame

        borderLayer.path = borderPath.cgPath
        borderLayer.frame = borderFrame
    }

    // MARK: - Notifications

    @objc func textDidBeginEditing() {
        updateAppearance()
    }

    @objc func textDidEndEditing() {
        updateAppearance()
    }

    // MARK: - Private

    private func setup() {
        backgroundColor = UIColor.clear

        borderLayer.lineWidth = borderWidth
        borderLayer.fillColor = UIColor.clear.cgColor
        backgroundLayer.mask = maskLayer

        layer.insertSublayer(borderLayer, at: 0)
        layer.insertSublayer(backgroundLayer, at: 0)

        updateAppearance()
        updateTextFieldEnabled()

        addTextFieldNotificationObservers()
    }

    private func addTextFieldNotificationObservers() {
        let notificationCenter = NotificationCenter.default

        notificationCenter.addObserver(self,
                                       selector: #selector(textDidBeginEditing),
                                       name: UITextField.textDidBeginEditingNotification,
                                       object: textField)
        notificationCenter.addObserver(self,
                                       selector: #selector(textDidEndEditing),
                                       name: UITextField.textDidEndEditingNotification,
                                       object: textField)
    }

    private func updateAppearance() {
        borderLayer.strokeColor = borderColor.cgColor
        backgroundLayer.backgroundColor = backgroundLayerColor.cgColor
        textField.textColor = textColor
    }

    private func updateTextFieldEnabled() {
        switch loginState {
        case .authenticating, .success:
            textField.isEnabled = false

        default:
            textField.isEnabled = true
        }
     }

    private func borderBezierPath(size: CGSize) -> UIBezierPath {
        let borderPath = UIBezierPath(roundedRect: CGRect(origin: .zero, size: size), cornerRadius: borderRadius)
        borderPath.lineWidth = borderWidth

        return borderPath
    }

    private func backgroundMaskImage(borderPath: UIBezierPath) -> UIImage {
        let renderer = UIGraphicsImageRenderer(bounds: borderPath.bounds)
        return renderer.image { (ctx) in
            borderPath.fill()

            // strip out any overlapping pixels between the border and the background
            borderPath.stroke(with: .clear, alpha: 0)
        }
    }
}
