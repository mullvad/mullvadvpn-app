package net.mullvad.mullvadvpn.lib.common.util

import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.GeographicLocationConstraint
import net.mullvad.mullvadvpn.model.LocationConstraint

fun LocationConstraint.toGeographicLocationConstraint(): GeographicLocationConstraint? =
    when (this) {
        is LocationConstraint.Location -> this.location
        is LocationConstraint.CustomList -> null
    }

fun Constraint<LocationConstraint>.toGeographicLocationConstraint():
    Constraint<GeographicLocationConstraint> =
    when (this) {
        is Constraint.Only ->
            when (value) {
                is LocationConstraint.Location ->
                    Constraint.Only((value as LocationConstraint.Location).location)
                is LocationConstraint.CustomList -> Constraint.Any()
            }
        is Constraint.Any -> Constraint.Any()
    }
