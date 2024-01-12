package net.mullvad.mullvadvpn.test.mockapi.constant

const val LOG_TAG = "mullvad-mockapi"

const val AUTH_TOKEN_URL_PATH = "/auth/v1/token"
const val DEVICES_URL_PATH = "/accounts/v1/devices"
const val ACCOUNT_URL_PATH = "/accounts/v1/accounts/me"
const val CREATE_ACCOUNT_URL_PATH = "/accounts/v1/accounts"

const val DUMMY_ID_1 = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"
const val DUMMY_DEVICE_NAME_1 = "mole mole"
const val DUMMY_ID_2 = "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb"
const val DUMMY_DEVICE_NAME_2 = "elom elom"
const val DUMMY_ID_3 = "cccccccc-cccc-cccc-cccc-cccccccccccc"
const val DUMMY_DEVICE_NAME_3 = "mole elom"
const val DUMMY_ID_4 = "dddddddd-dddd-dddd-dddd-dddddddddddd"
const val DUMMY_DEVICE_NAME_4 = "yellow hat"
const val DUMMY_ID_5 = "eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee"
const val DUMMY_DEVICE_NAME_5 = "secure tunnel"
const val DUMMY_ID_6 = "ffffffff-ffff-ffff-ffff-ffffffffffff"
const val DUMMY_DEVICE_NAME_6 = "red lobster"
const val DUMMY_ACCESS_TOKEN =
    "mva_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"

val DEFAULT_DEVICE_LIST = mapOf(DUMMY_ID_1 to DUMMY_DEVICE_NAME_1)
val FULL_DEVICE_LIST =
    mapOf(
        DUMMY_ID_1 to DUMMY_DEVICE_NAME_1,
        DUMMY_ID_2 to DUMMY_DEVICE_NAME_2,
        DUMMY_ID_3 to DUMMY_DEVICE_NAME_3,
        DUMMY_ID_4 to DUMMY_DEVICE_NAME_4,
        DUMMY_ID_5 to DUMMY_DEVICE_NAME_5
    )
