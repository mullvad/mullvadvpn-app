import { useCallback } from 'react';

import { urls } from '../../../../../../shared/constants';
import { messages } from '../../../../../../shared/gettext';
import { useAppContext } from '../../../../../context';
import { useSelector } from '../../../../../redux/store';
import { AriaDescribed, AriaDescription, AriaDescriptionGroup } from '../../../../AriaGroup';
import * as Cell from '../../../../cell';

export function FaqButton() {
  const isOffline = useSelector((state) => state.connection.isBlocked);
  const { openUrl } = useAppContext();

  const openFaq = useCallback(() => openUrl(urls.faq), [openUrl]);

  return (
    <AriaDescriptionGroup>
      <AriaDescribed>
        <Cell.CellButton disabled={isOffline} onClick={openFaq}>
          <Cell.Label>
            {
              // TRANSLATORS: Link to the webpage
              messages.pgettext('support-view', 'FAQs & Guides')
            }
          </Cell.Label>
          <AriaDescription>
            <Cell.CellTintedIcon
              icon="external"
              aria-label={messages.pgettext('accessibility', 'Opens externally')}
            />
          </AriaDescription>
        </Cell.CellButton>
      </AriaDescribed>
    </AriaDescriptionGroup>
  );
}
