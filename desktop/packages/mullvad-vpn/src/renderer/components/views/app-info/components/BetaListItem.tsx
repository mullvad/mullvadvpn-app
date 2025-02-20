import React from 'react';

import { messages } from '../../../../../shared/gettext';
import { useAppContext } from '../../../../context';
import { ListItem } from '../../../../lib/components/list-item';
import { useSelector } from '../../../../redux/store';
import Switch from '../../../Switch';

export const BetaListItem = () => {
  const isBeta = useSelector((state) => state.version.isBeta);
  const showBetaReleases = useSelector((state) => state.settings.showBetaReleases);
  const { setShowBetaReleases } = useAppContext();
  const switchId = React.useId();
  const labelId = React.useId();
  const descriptionId = React.useId();

  return (
    <ListItem disabled={isBeta}>
      <ListItem.Item>
        <ListItem.Content>
          <ListItem.Label id={labelId} as={'label'} htmlFor={switchId}>
            {
              // TRANSLATORS: Label for switch to toggle beta program.
              messages.pgettext('app-info-view', 'Beta program')
            }
          </ListItem.Label>
          <Switch
            id={switchId}
            aria-labelledby={labelId}
            aria-describedby={descriptionId}
            isOn={showBetaReleases}
            onChange={setShowBetaReleases}
          />
        </ListItem.Content>
      </ListItem.Item>
      <ListItem.Footer>
        <ListItem.Text id={descriptionId}>
          {isBeta
            ? // TRANSLATORS: Description for beta program switch when using a beta version.
              messages.pgettext(
                'support-view',
                'This option is unavailable while using a beta version.',
              )
            : // TRANSLATORS: Description for beta program switch.
              messages.pgettext(
                'support-view',
                'Enable to get notified when new beta versions of the app are released.',
              )}
        </ListItem.Text>
      </ListItem.Footer>
    </ListItem>
  );
};
