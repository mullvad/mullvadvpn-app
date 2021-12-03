import { formatAccountToken } from '../lib/account';
import ClipboardLabel from './ClipboardLabel';

interface IAccountTokenLabelProps {
  accountToken: string;
  obscureValue?: boolean;
  className?: string;
}

export default function AccountTokenLabel(props: IAccountTokenLabelProps) {
  return (
    <ClipboardLabel
      value={props.accountToken}
      displayValue={formatAccountToken(props.accountToken)}
      obscureValue={props.obscureValue}
      className={props.className}
    />
  );
}
