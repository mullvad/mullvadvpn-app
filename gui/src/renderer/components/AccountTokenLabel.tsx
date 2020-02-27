import * as React from 'react';
import { Types } from 'reactxp';
import { formatAccountToken } from '../lib/account';
import ClipboardLabel from './ClipboardLabel';

interface IAccountTokenLabelProps {
  accountToken: string;
  style?: Types.StyleRuleSetRecursive<Types.TextStyleRuleSet>;
  messageStyle?: Types.StyleRuleSetRecursive<Types.TextStyleRuleSet>;
}

export default function AccountTokenLabel(props: IAccountTokenLabelProps) {
  return (
    <ClipboardLabel
      style={props.style}
      messageStyle={props.messageStyle}
      value={props.accountToken}
      displayValue={formatAccountToken(props.accountToken)}
    />
  );
}
