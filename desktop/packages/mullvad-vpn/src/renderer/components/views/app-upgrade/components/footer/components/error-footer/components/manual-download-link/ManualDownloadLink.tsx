import { messages } from '../../../../../../../../../../shared/gettext';
import { ExternalLink } from '../../../../../../../../../components/ExternalLink';
import { useDownloadUrl } from './hooks';

export function ManualDownloadLink() {
  const downloadUrl = useDownloadUrl();

  return (
    <ExternalLink variant="labelTiny" to={downloadUrl}>
      {
        // TRANSLATORS: Link shown to optionally manually download the update
        // TRANSLATORS: due to repeated errors in the upgrade process.
        messages.pgettext(
          'app-upgrade-view',
          'Having problems? Try downloading the app from our website',
        )
      }
      <ExternalLink.Icon
        aria-description={messages.pgettext('accessibility', 'Opens externally')}
        icon="external"
      />
    </ExternalLink>
  );
}
