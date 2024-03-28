import { BridgeSettings, wrapConstraint } from "../../shared/daemon-rpc-types";
import { BridgeSettingsRedux } from "../redux/settings/reducers";

export function convertToBridgeSettings(bridgeSettings: BridgeSettingsRedux): BridgeSettings {
  return {
    ...bridgeSettings,
    normal: {
      ...bridgeSettings.normal,
      location: wrapConstraint(bridgeSettings.normal.location),
    }
  };
}
