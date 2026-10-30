import React from 'react';

import { MultihopMode } from '../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../shared/gettext';
import { SettingsListbox } from '../../../../components/settings-listbox';
import useActions from '../../../../lib/actionsHook';
import { useNormalRelaySettings } from '../../../../lib/relay-settings-hooks';
import userInterface from '../../../../redux/userinterface/actions';
import { useActiveFilters, useEntryLocation } from '../../../locations/hooks';
import { LocationType } from '../../../locations/types';
import { useMultihop } from '../../hooks';
import { EntryFiltersWarningDialog } from '../entry-filters-warning-dialog';
import { AlwaysOption, NeverOption, WhenNeededOption } from './components';

export function MultihopSetting() {
  const { multihop, setMultihop } = useMultihop();
  const { setSelectLocationView } = useActions(userInterface);
  const { isAnyListFilterActive: isAnyEntryListFilterActive } = useActiveFilters(
    LocationType.entry,
  );
  const [dialogOpen, setDialogOpen] = React.useState(false);
  const { entryLocation } = useEntryLocation();

  const normalRelaySettings = useNormalRelaySettings();
  const unavailable = normalRelaySettings === null;

  const handleValueChange = React.useCallback(
    async (multihop: MultihopMode) => {
      if (multihop === 'when-needed' && entryLocation !== 'any' && isAnyEntryListFilterActive) {
        setDialogOpen(true);
      } else {
        await setMultihop({ multihop });
        setSelectLocationView(LocationType.exit);
      }
    },
    [entryLocation, isAnyEntryListFilterActive, setMultihop, setSelectLocationView],
  );

  const handleContinue = React.useCallback(async () => {
    await setMultihop({ multihop: 'when-needed' });
    setSelectLocationView(LocationType.exit);
  }, [setMultihop, setSelectLocationView]);

  const handleCancel = React.useCallback(() => setDialogOpen(false), [setDialogOpen]);

  return (
    <>
      <SettingsListbox
        anchorId="multihop-setting"
        value={multihop}
        onValueChange={handleValueChange}>
        <SettingsListbox.Header>
          <SettingsListbox.Header.Item>
            <SettingsListbox.Header.Item.Label>
              {
                // TRANSLATORS: Title for Multihop mode setting.
                messages.pgettext('wireguard-settings-view', 'Mode')
              }
            </SettingsListbox.Header.Item.Label>
          </SettingsListbox.Header.Item>
        </SettingsListbox.Header>
        <SettingsListbox.Options>
          <WhenNeededOption disabled={unavailable} />
          <AlwaysOption disabled={unavailable} />
          <NeverOption disabled={unavailable} />
        </SettingsListbox.Options>
      </SettingsListbox>
      <EntryFiltersWarningDialog
        open={dialogOpen}
        onCancel={handleCancel}
        onContinue={handleContinue}
      />
    </>
  );
}
