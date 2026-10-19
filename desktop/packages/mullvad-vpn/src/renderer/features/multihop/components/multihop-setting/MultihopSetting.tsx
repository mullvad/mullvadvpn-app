import React from 'react';
import styled from 'styled-components';

import { MultihopMode } from '../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../shared/gettext';
import { Info } from '../../../../components/info';
import { SettingsListbox } from '../../../../components/settings-listbox';
import useActions from '../../../../lib/actionsHook';
import { Icon } from '../../../../lib/components';
import { formatHtml } from '../../../../lib/html-formatter';
import { useNormalRelaySettings } from '../../../../lib/relay-settings-hooks';
import userInterface from '../../../../redux/userinterface/actions';
import { LocationType } from '../../../locations/types';
import { useMultihop } from '../../hooks';

const StyledIcon = styled(Icon)`
  display: inline-block;
  vertical-align: middle;
`;

export function MultihopSetting() {
  const { multihop, setMultihop } = useMultihop();
  const { setSelectLocationView } = useActions(userInterface);

  const normalRelaySettings = useNormalRelaySettings();
  const unavailable = normalRelaySettings === null;

  const handleValueChange = React.useCallback(
    async (multihop: MultihopMode) => {
      await setMultihop({ multihop });
      setSelectLocationView(LocationType.exit);
    },
    [setMultihop, setSelectLocationView],
  );

  const [dialogOpen, setDialogOpen] = React.useState(false);

  const openDialog = React.useCallback(() => setDialogOpen(true), []);

  return (
    <SettingsListbox anchorId="multihop-setting" value={multihop} onValueChange={handleValueChange}>
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
        <SettingsListbox.Options.SplitOption value="when-needed" disabled={unavailable}>
          <SettingsListbox.Options.SplitOption.Item>
            <SettingsListbox.Options.SplitOption.Label>
              {
                // TRANSLATORS: Label for the Multihop mode setting's option to use the multihop
                // TRANSLATORS: feature when it is needed.
                messages.pgettext('wireguard-settings-view', 'When needed')
              }
            </SettingsListbox.Options.SplitOption.Label>
          </SettingsListbox.Options.SplitOption.Item>
          <SettingsListbox.Options.SplitOption.InfoButton onClick={openDialog} />
          <Info open={dialogOpen} onOpenChange={setDialogOpen}>
            <Info.Dialog>
              <Info.Dialog.Text>
                {
                  // TRANSLATORS: Explanation for the "When needed" option in the Multihop mode setting.
                  messages.pgettext(
                    'wireguard-settings-view',
                    'To ensure your current settings work with your selected location, and to avoid blocking your connection, the app might automatically multihop via a different entry server.',
                  )
                }
              </Info.Dialog.Text>
              <Info.Dialog.Text>
                {formatHtml(
                  // TRANSLATORS: Explanation for the symbol indicating automatic multihop.
                  // Available placeholders:
                  // TRANSLATORS: <img></img> - Will be replaced by an icon.
                  messages.pgettext(
                    'wireguard-settings-view',
                    'This will be indicated by the <img></img> symbol.',
                  ),
                  {
                    img: () => (
                      <StyledIcon
                        icon="magic-multihop"
                        aria-label={messages.gettext('Magic multihop')}
                        color="whiteAlpha60"
                        size="tiny"
                      />
                    ),
                  },
                )}
              </Info.Dialog.Text>
              <Info.Dialog.Text>
                {
                  // TRANSLATORS: Warning about ignoring filter settings for the automatically selected entry server.
                  messages.pgettext(
                    'wireguard-settings-view',
                    'Attention: This will ignore filter settings for the entry server that is being automatically selected.',
                  )
                }
              </Info.Dialog.Text>
            </Info.Dialog>
          </Info>
        </SettingsListbox.Options.SplitOption>
        <SettingsListbox.Options.BaseOption value="always" disabled={unavailable}>
          {
            // TRANSLATORS: Label for the Multihop mode setting's option to always use the multihop
            // TRANSLATORS: feature.
            messages.pgettext('wireguard-settings-view', 'Always')
          }
        </SettingsListbox.Options.BaseOption>
        <SettingsListbox.Options.BaseOption value="never" disabled={unavailable}>
          {
            // TRANSLATORS: Label for the Multihop mode setting's option to never use the multihop
            // TRANSLATORS: feature.
            messages.pgettext('wireguard-settings-view', 'Never')
          }
        </SettingsListbox.Options.BaseOption>
      </SettingsListbox.Options>
    </SettingsListbox>
  );
}
