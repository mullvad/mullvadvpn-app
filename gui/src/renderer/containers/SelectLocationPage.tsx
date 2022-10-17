import { useCallback, useMemo } from 'react';

import BridgeSettingsBuilder from '../../shared/bridge-settings-builder';
import { LiftedConstraint, Ownership, RelayLocation } from '../../shared/daemon-rpc-types';
import log from '../../shared/logging';
import RelaySettingsBuilder from '../../shared/relay-settings-builder';
import SelectLocation from '../components/SelectLocation';
import { useAppContext } from '../context';
import { createWireguardRelayUpdater } from '../lib/constraint-updater';
import filterLocations from '../lib/filter-locations';
import { useHistory } from '../lib/history';
import { RoutePath } from '../lib/routes';
import { useSelector } from '../redux/store';

export default function SelectLocationPage() {
  const history = useHistory();

  const { updateRelaySettings, connectTunnel, updateBridgeSettings } = useAppContext();

  const locale = useSelector((state) => state.userInterface.locale);
  const settings = useSelector((state) => state.settings);
  const { relaySettings, bridgeSettings, bridgeState } = settings;

  const providers = useMemo(
    () => ('normal' in relaySettings ? relaySettings.normal.providers : []),
    [relaySettings],
  );

  const ownership = useMemo(
    () => ('normal' in relaySettings ? relaySettings.normal.ownership : Ownership.any),
    [relaySettings],
  );

  const tunnelProtocol = useMemo(
    () => ('normal' in relaySettings ? relaySettings.normal.tunnelProtocol : 'any'),
    [relaySettings],
  );

  const selectedExitLocation = useMemo<RelayLocation | undefined>(() => {
    if ('normal' in relaySettings) {
      const exitLocation = relaySettings.normal.location;
      if (exitLocation !== 'any') {
        return exitLocation;
      }
    }
    return undefined;
  }, [relaySettings]);

  const selectedBridgeLocation = useMemo<LiftedConstraint<RelayLocation> | undefined>(() => {
    return tunnelProtocol === 'openvpn' && 'normal' in bridgeSettings
      ? bridgeSettings.normal.location
      : undefined;
  }, [tunnelProtocol, bridgeSettings]);

  const multihopEnabled = useMemo(() => {
    return (
      tunnelProtocol !== 'openvpn' &&
      'normal' in relaySettings &&
      relaySettings.normal.wireguard.useMultihop
    );
  }, [tunnelProtocol, relaySettings]);

  const selectedEntryLocation = useMemo<RelayLocation | undefined>(() => {
    if (multihopEnabled && 'normal' in relaySettings) {
      const entryLocation = relaySettings.normal.wireguard.entryLocation;
      if (multihopEnabled && entryLocation !== 'any') {
        return entryLocation;
      }
    }
    return undefined;
  }, [relaySettings, multihopEnabled]);

  const allowEntrySelection = useMemo(() => {
    return (
      (tunnelProtocol === 'openvpn' && bridgeState === 'on') ||
      ((tunnelProtocol === 'any' || tunnelProtocol === 'wireguard') && multihopEnabled)
    );
  }, [tunnelProtocol, bridgeState, multihopEnabled]);

  const relayLocations = filterLocations(settings.relayLocations, providers, ownership);
  const bridgeLocations = filterLocations(settings.bridgeLocations, providers, ownership);

  const onClose = useCallback(() => history.dismiss(), [history]);
  const onViewFilter = useCallback(() => history.push(RoutePath.filter), [history]);
  const onSelectExitLocation = useCallback(
    async (relayLocation: RelayLocation) => {
      // dismiss the view first
      history.dismiss();
      try {
        const relayUpdate = RelaySettingsBuilder.normal().location.fromRaw(relayLocation).build();

        await updateRelaySettings(relayUpdate);
        await connectTunnel();
      } catch (e) {
        const error = e as Error;
        log.error(`Failed to select the exit location: ${error.message}`);
      }
    },
    [connectTunnel, updateRelaySettings, history],
  );
  const onSelectEntryLocation = useCallback(
    async (entryLocation: RelayLocation) => {
      // dismiss the view first
      history.dismiss();

      const relayUpdate = createWireguardRelayUpdater(relaySettings)
        .tunnel.wireguard((wireguard) => wireguard.entryLocation.exact(entryLocation))
        .build();

      try {
        await updateRelaySettings(relayUpdate);
      } catch (e) {
        const error = e as Error;
        log.error('Failed to select the entry location', error.message);
      }
    },
    [history, relaySettings, updateRelaySettings],
  );
  const onSelectBridgeLocation = useCallback(
    async (bridgeLocation: RelayLocation) => {
      // dismiss the view first
      history.dismiss();

      try {
        await updateBridgeSettings(
          new BridgeSettingsBuilder().location.fromRaw(bridgeLocation).build(),
        );
      } catch (e) {
        const error = e as Error;
        log.error(`Failed to select the bridge location: ${error.message}`);
      }
    },
    [history, updateBridgeSettings],
  );
  const onSelectClosestToExit = useCallback(async () => {
    history.dismiss();

    try {
      await updateBridgeSettings(new BridgeSettingsBuilder().location.any().build());
    } catch (e) {
      const error = e as Error;
      log.error(`Failed to set the bridge location to closest to exit: ${error.message}`);
    }
  }, [updateBridgeSettings, history]);

  const onClearProviders = useCallback(async () => {
    await updateRelaySettings({ normal: { providers: [] } });
  }, [updateRelaySettings]);

  const onClearOwnership = useCallback(async () => {
    await updateRelaySettings({ normal: { ownership: Ownership.any } });
  }, [updateRelaySettings]);

  return (
    <SelectLocation
      locale={locale}
      selectedExitLocation={selectedExitLocation}
      selectedEntryLocation={selectedEntryLocation}
      selectedBridgeLocation={selectedBridgeLocation}
      relayLocations={relayLocations}
      bridgeLocations={bridgeLocations}
      allowEntrySelection={allowEntrySelection}
      tunnelProtocol={tunnelProtocol}
      providers={providers}
      ownership={ownership}
      onClose={onClose}
      onViewFilter={onViewFilter}
      onSelectExitLocation={onSelectExitLocation}
      onSelectEntryLocation={onSelectEntryLocation}
      onSelectBridgeLocation={onSelectBridgeLocation}
      onSelectClosestToExit={onSelectClosestToExit}
      onClearProviders={onClearProviders}
      onClearOwnership={onClearOwnership}
    />
  );
}
