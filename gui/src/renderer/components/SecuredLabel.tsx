import styled from 'styled-components';

import { colors } from '../../config.json';
import { messages } from '../../shared/gettext';

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
  [SecuredDisplayStyle.securing]: colors.white,
  [SecuredDisplayStyle.securingPq]: colors.white,
  [SecuredDisplayStyle.unsecuring]: colors.white,
  [SecuredDisplayStyle.secured]: colors.green,
  [SecuredDisplayStyle.securedPq]: colors.green,
  [SecuredDisplayStyle.blocked]: colors.white,
  [SecuredDisplayStyle.unsecured]: colors.red,
  [SecuredDisplayStyle.failedToSecure]: colors.red,
};

const StyledSecuredLabel = styled.span((props: { displayStyle: SecuredDisplayStyle }) => ({
  display: 'inline-block',
  minHeight: '22px',
  color: securedDisplayStyleColorMap[props.displayStyle],
}));

interface ISecuredLabelProps {
  displayStyle: SecuredDisplayStyle;
  className?: string;
}

export default function SecuredLabel(props: ISecuredLabelProps) {
  return (
    <StyledSecuredLabel {...props} role="status" aria-live="polite">
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
