import * as React from 'react';
import styled from 'styled-components';
import { colors } from '../../config.json';
import { messages } from '../../shared/gettext';

export enum SecuredDisplayStyle {
  secured,
  blocked,
  securing,
  unsecured,
  failedToSecure,
}

const securedDisplayStyleColorMap = {
  [SecuredDisplayStyle.securing]: colors.white,
  [SecuredDisplayStyle.secured]: colors.green,
  [SecuredDisplayStyle.blocked]: colors.green,
  [SecuredDisplayStyle.unsecured]: colors.red,
  [SecuredDisplayStyle.failedToSecure]: colors.red,
};

const securedDisplayStyleTextMap = {
  [SecuredDisplayStyle.securing]: messages.gettext('CREATING SECURE CONNECTION'),
  [SecuredDisplayStyle.secured]: messages.gettext('SECURE CONNECTION'),
  [SecuredDisplayStyle.blocked]: messages.gettext('BLOCKED CONNECTION'),
  [SecuredDisplayStyle.unsecured]: messages.gettext('UNSECURED CONNECTION'),
  [SecuredDisplayStyle.failedToSecure]: messages.gettext('FAILED TO SECURE CONNECTION'),
};

const StyledSecuredLabel = styled.span((props: { displayStyle: SecuredDisplayStyle }) => ({
  color: securedDisplayStyleColorMap[props.displayStyle],
}));

interface ISecuredLabelProps {
  displayStyle: SecuredDisplayStyle;
  className?: string;
}

export default function SecuredLabel(props: ISecuredLabelProps) {
  const text = securedDisplayStyleTextMap[props.displayStyle];
  return <StyledSecuredLabel {...props}>{text}</StyledSecuredLabel>;
}
