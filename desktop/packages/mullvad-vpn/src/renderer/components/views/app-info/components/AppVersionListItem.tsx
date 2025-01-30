import { useCallback } from 'react';

import { messages } from '../../../../../shared/gettext';
import { getDownloadUrl } from '../../../../../shared/version';
import { useAppContext } from '../../../../context';
import { Colors } from '../../../../lib/foundations';
import { useSelector } from '../../../../redux/store';
import * as Cell from '../../../cell';
import { LabelStack } from '../../../Layout';

export function AppVersionListItem() {
  const appVersion = useSelector((state) => state.version.current);
  const consistentVersion = useSelector((state) => state.version.consistent);
  const upToDateVersion = useSelector((state) => (state.version.suggestedUpgrade ? false : true));
  const suggestedIsBeta = useSelector((state) => state.version.suggestedIsBeta ?? false);
  const isOffline = useSelector((state) => state.connection.isBlocked);

  const { openUrl } = useAppContext();
  const openDownloadLink = useCallback(
    () => openUrl(getDownloadUrl(suggestedIsBeta)),
    [openUrl, suggestedIsBeta],
  );

  let alertIcon;
  let footer;
  if (!consistentVersion || !upToDateVersion) {
    const inconsistentVersionMessage = messages.pgettext(
      'app-info-view',
      'App is out of sync. Please quit and restart.',
    );

    const updateAvailableMessage = messages.pgettext(
      'app-info-view',
      'Update available. Install the latest app version to stay up to date.',
    );

    const message = !consistentVersion ? inconsistentVersionMessage : updateAvailableMessage;

    alertIcon = <Cell.CellIcon icon="alert-circle" color={Colors.red} />;
    footer = (
      <Cell.CellFooter>
        <Cell.CellFooterText>{message}</Cell.CellFooterText>
      </Cell.CellFooter>
    );
  }

  return (
    <>
      <Cell.CellNavigationButton
        disabled={isOffline}
        onClick={openDownloadLink}
        icon={{
          icon: 'external',
          'aria-label': messages.pgettext('accessibility', 'Opens externally'),
        }}>
        <LabelStack>
          {alertIcon}
          <Cell.Label>{messages.pgettext('app-info-view', 'App version')}</Cell.Label>
        </LabelStack>
        <Cell.SubText>{appVersion}</Cell.SubText>
      </Cell.CellNavigationButton>
      {footer}
    </>
  );
}
