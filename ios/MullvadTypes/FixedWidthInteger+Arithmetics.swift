//
//  FixedWidthInteger+Arithmetics.swift
//  PacketTunnel
//
//  Created by pronebird on 28/08/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension FixedWidthInteger {
    /// Saturating integer multiplication. Computes `self * rhs`, saturating at the numeric bounds
    /// instead of overflowing.
    public func saturatingMultiplication(_ rhs: Self) -> Self {
        let (partialValue, isOverflow) = multipliedReportingOverflow(by: rhs)

        if isOverflow {
            return signum() == rhs.signum() ? .max : .min
        } else {
            return partialValue
        }
    }

    /// Saturating integer addition. Computes `self + rhs`, saturating at the numeric bounds
    /// instead of overflowing.
    public func saturatingAddition(_ rhs: Self) -> Self {
        let (partialValue, isOverflow) = addingReportingOverflow(rhs)

        if isOverflow {
            return partialValue.signum() >= 0 ? .min : .max
        } else {
            return partialValue
        }
    }

    /// Saturating integer subtraction. Computes `self - rhs`, saturating at the numeric bounds
    /// instead of overflowing.
    public func saturatingSubtraction(_ rhs: Self) -> Self {
        let (partialValue, isOverflow) = subtractingReportingOverflow(rhs)

        if isOverflow {
            return partialValue.signum() >= 0 ? .min : .max
        } else {
            return partialValue
        }
    }

    /// Saturating integer exponentiation. Computes `self ** exp`, saturating at the numeric
    /// bounds instead of overflowing.
    public func saturatingPow(_ exp: UInt32) -> Self {
        let result = pow(Double(self), Double(exp))

        if result.isFinite {
            if result <= Double(Self.min) {
                return .min
            } else if result >= Double(Self.max) {
                return .max
            } else {
                return Self(result)
            }
        } else {
            return result.sign == .minus ? Self.min : Self.max
        }
    }
}
