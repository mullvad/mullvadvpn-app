import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../shared/gettext';
import { LabelTinySemiBold } from '../../../../../lib/components';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { formatHtml } from '../../../../../lib/html-formatter';
import { useSelectLocationViewContext } from '../../SelectLocationViewContext';

export function NoSearchResult() {
  const { searchTerm } = useSelectLocationViewContext();
  return (
    <FlexColumn justifyContent="center" alignItems="center">
      <LabelTinySemiBold color="whiteAlpha60" textAlign="center">
        {formatHtml(
          sprintf(messages.gettext('No result for <b>%(searchTerm)s</b>.'), {
            searchTerm,
          }),
        )}
      </LabelTinySemiBold>
      <LabelTinySemiBold color="whiteAlpha60" textAlign="center">
        {messages.gettext('Try a different search.')}
      </LabelTinySemiBold>
    </FlexColumn>
  );
}
