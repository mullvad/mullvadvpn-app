import React from 'react';

import { messages } from '../../../../../shared/gettext';
import { IconButton, type IconButtonProps } from '../../../../lib/components';
import { useInfoContext } from '../../InfoContext';

export type InfoButtonProps = IconButtonProps;

export function InfoButton(props: InfoButtonProps) {
  const { onOpenChange } = useInfoContext();

  const handleClick = React.useCallback(() => {
    onOpenChange(true);
  }, [onOpenChange]);

  return (
    <IconButton
      onClick={handleClick}
      aria-label={messages.pgettext('accessibility', 'More information')}
      {...props}>
      <IconButton.Icon icon="info-circle" />
    </IconButton>
  );
}
