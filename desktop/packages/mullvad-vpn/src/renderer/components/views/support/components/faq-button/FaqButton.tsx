import { useCallback } from 'react';

import { urls } from '../../../../../../shared/constants';
import { messages } from '../../../../../../shared/gettext';
import { useAppContext } from '../../../../../context';
import { ListItem } from '../../../../../lib/components/list-item';
import { useSelector } from '../../../../../redux/store';

export function FaqButton() {
  const isOffline = useSelector((state) => state.connection.isBlocked);
  const { openUrl } = useAppContext();

  const openFaq = useCallback(() => openUrl(urls.faq), [openUrl]);

  return (
    <ListItem disabled={isOffline}>
      <ListItem.Trigger
        onClick={openFaq}
        aria-description={messages.pgettext('accessibility', 'Opens externally')}>
        <ListItem.Item>
          <ListItem.Item.Label>
            {
              // TRANSLATORS: Link to the webpage
              messages.pgettext('support-view', 'FAQs & Guides')
            }
          </ListItem.Item.Label>
          <ListItem.Item.ActionGroup>
            <ListItem.Item.Icon icon="external" />
          </ListItem.Item.ActionGroup>
        </ListItem.Item>
      </ListItem.Trigger>
    </ListItem>
  );
}
