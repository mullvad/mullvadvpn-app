import styled from 'styled-components';

import { messages } from '../../shared/gettext';
import { Colors } from '../lib/foundations';

export enum SecuredDisplayStyle {
  secured,
  securedPq,
  blocked,
  securing,
  securingPq,
  unsecured,
  unsecuring,
  failedToSecure,
}

const securedDisplayStyleColorMap = {
  [SecuredDisplayStyle.securing]: Colors.white,
  [SecuredDisplayStyle.securingPq]: Colors.white,
  [SecuredDisplayStyle.unsecuring]: Colors.white,
  [SecuredDisplayStyle.secured]: Colors.green,
  [SecuredDisplayStyle.securedPq]: Colors.green,
  [SecuredDisplayStyle.blocked]: Colors.white,
  [SecuredDisplayStyle.unsecured]: Colors.red,
  [SecuredDisplayStyle.failedToSecure]: Colors.red,
};

const StyledSecuredLabel = styled.span<{ $displayStyle: SecuredDisplayStyle }>((props) => ({
  display: 'inline-block',
  minHeight: '22px',
  color: securedDisplayStyleColorMap[props.$displayStyle],
}));

interface ISecuredLabelProps {
  displayStyle: SecuredDisplayStyle;
  className?: string;
}

export default function SecuredLabel(props: ISecuredLabelProps) {
  const { displayStyle, ...otherProps } = props;
  return (
    <StyledSecuredLabel
      $displayStyle={displayStyle}
      {...otherProps}
      role="status"
      aria-live="polite">
      {getLabelText(props.displayStyle)}
    </StyledSecuredLabel>
  );
}

function getLabelText(displayStyle: SecuredDisplayStyle) {
  switch (displayStyle) {
    case SecuredDisplayStyle.secured:
      return messages.gettext('SECURE CONNECTION');

    case SecuredDisplayStyle.securedPq:
      // TRANSLATORS: The connection is secure and isn't breakable by quantum computers.
      return messages.gettext('QUANTUM SECURE CONNECTION');

    case SecuredDisplayStyle.blocked:
      return messages.gettext('BLOCKED CONNECTION');

    case SecuredDisplayStyle.securing:
      return messages.gettext('CREATING SECURE CONNECTION');

    case SecuredDisplayStyle.securingPq:
      // TRANSLATORS: Creating a secure connection that isn't breakable by quantum computers.
      return messages.gettext('CREATING QUANTUM SECURE CONNECTION');

    case SecuredDisplayStyle.unsecured:
      return messages.gettext('UNSECURED CONNECTION');

    case SecuredDisplayStyle.unsecuring:
      return '';

    case SecuredDisplayStyle.failedToSecure:
      return messages.gettext('FAILED TO SECURE CONNECTION');
  }
}
