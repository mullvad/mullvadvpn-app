//
//  SpinnerActivityIndicatorView.swift
//  MullvadVPN
//
//  Created by pronebird on 15/05/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import UIKit

private let kRotationAnimationKey = "rotation"
private let kAnimationDuration = 0.6

class SpinnerActivityIndicatorView: UIView {
    enum Style {
        case small, medium, large

        var intrinsicSize: CGSize {
            switch self {
            case .small:
                return .init(width: 16, height: 16)
            case .medium:
                return .init(width: 20, height: 20)
            case .large:
                return .init(width: 60, height: 60)
            }
        }
    }

    private let imageView = UIImageView(image: UIImage(named: "IconSpinner"))

    private(set) var isAnimating = false
    private(set) var style = Style.large

    private var startTime = CFTimeInterval(0)
    private var stopTime = CFTimeInterval(0)

    override var intrinsicContentSize: CGSize {
        return style.intrinsicSize
    }

    init(style: Style) {
        self.style = style

        super.init(frame: CGRect(origin: .zero, size: style.intrinsicSize))

        addSubview(imageView)
        isHidden = true
        backgroundColor = UIColor.clear

        NotificationCenter.default.addObserver(
            self,
            selector: #selector(restartAnimationIfNeeded),
            name: UIApplication.willEnterForegroundNotification,
            object: nil
        )
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func didMoveToWindow() {
        super.didMoveToWindow()

        if window != nil {
            restartAnimationIfNeeded()
        }
    }

    override func layoutSubviews() {
        super.layoutSubviews()

        imageView.frame = CGRect(origin: .zero, size: style.intrinsicSize)
    }

    func startAnimating() {
        guard !isAnimating else { return }
        isAnimating = true

        isHidden = false
        addAnimation()
    }

    func stopAnimating() {
        guard isAnimating else { return }
        isAnimating = false

        isHidden = true
        removeAnimation()
    }

    private func addAnimation() {
        let timeOffset = stopTime - startTime

        let anim = animation()
        anim.timeOffset = timeOffset

        layer.add(anim, forKey: kRotationAnimationKey)

        startTime = layer.convertTime(CACurrentMediaTime(), from: nil) - timeOffset
    }

    private func removeAnimation() {
        layer.removeAnimation(forKey: kRotationAnimationKey)

        stopTime = layer.convertTime(CACurrentMediaTime(), from: nil)
    }

    @objc private func restartAnimationIfNeeded() {
        let anim = layer.animation(forKey: kRotationAnimationKey)

        if isAnimating && anim == nil {
            removeAnimation()
            addAnimation()
        }
    }

    private func animation() -> CABasicAnimation {
        let animation = CABasicAnimation(keyPath: "transform.rotation")
        animation.toValue = NSNumber(value: Double.pi * 2)
        animation.duration = kAnimationDuration
        animation.repeatCount = Float.infinity
        animation.timingFunction = CAMediaTimingFunction(name: .linear)

        return animation
    }
}
