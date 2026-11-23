package net.mullvad.mullvadvpn.lib.userpreferences

enum class OnboardingCheckpoint {
    NOT_STARTED,
    WELCOME,
    NOTIFICATIONS,
    GOOD_TO_KNOW,
    PART_1_COMPLETED,
    PREFERRED_SETTINGS,
    COMPLETED,
}

internal fun OnboardingCheckpointProto.toDomain(): OnboardingCheckpoint =
    when (this) {
        OnboardingCheckpointProto.UNRECOGNIZED,
        OnboardingCheckpointProto.ONBOARDING_CHECKPOINT_NOT_STARTED ->
            OnboardingCheckpoint.NOT_STARTED
        OnboardingCheckpointProto.ONBOARDING_CHECKPOINT_WELCOME -> OnboardingCheckpoint.WELCOME
        OnboardingCheckpointProto.ONBOARDING_CHECKPOINT_NOTIFICATIONS ->
            OnboardingCheckpoint.NOTIFICATIONS
        OnboardingCheckpointProto.ONBOARDING_CHECKPOINT_GOOD_TO_KNOW ->
            OnboardingCheckpoint.GOOD_TO_KNOW
        OnboardingCheckpointProto.ONBOARDING_CHECKPOINT_PART_1_COMPLETED ->
            OnboardingCheckpoint.PART_1_COMPLETED
        OnboardingCheckpointProto.ONBOARDING_CHECKPOINT_PREFERRED_SETTINGS ->
            OnboardingCheckpoint.PREFERRED_SETTINGS
        OnboardingCheckpointProto.ONBOARDING_CHECKPOINT_COMPLETED -> OnboardingCheckpoint.COMPLETED
    }

internal fun OnboardingCheckpoint.toProto(): OnboardingCheckpointProto =
    when (this) {
        OnboardingCheckpoint.NOT_STARTED ->
            OnboardingCheckpointProto.ONBOARDING_CHECKPOINT_NOT_STARTED
        OnboardingCheckpoint.WELCOME -> OnboardingCheckpointProto.ONBOARDING_CHECKPOINT_WELCOME
        OnboardingCheckpoint.NOTIFICATIONS ->
            OnboardingCheckpointProto.ONBOARDING_CHECKPOINT_NOTIFICATIONS
        OnboardingCheckpoint.GOOD_TO_KNOW ->
            OnboardingCheckpointProto.ONBOARDING_CHECKPOINT_GOOD_TO_KNOW
        OnboardingCheckpoint.PART_1_COMPLETED ->
            OnboardingCheckpointProto.ONBOARDING_CHECKPOINT_PART_1_COMPLETED
        OnboardingCheckpoint.PREFERRED_SETTINGS ->
            OnboardingCheckpointProto.ONBOARDING_CHECKPOINT_PREFERRED_SETTINGS
        OnboardingCheckpoint.COMPLETED -> OnboardingCheckpointProto.ONBOARDING_CHECKPOINT_COMPLETED
    }
