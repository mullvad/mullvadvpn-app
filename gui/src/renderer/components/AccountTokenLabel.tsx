import { formatAccountToken } from '../lib/account';
import ClipboardLabel from './ClipboardLabel';

interface IAccountTokenLabelProps {
  accountToken: string;
  className?: string;
}

export default function AccountTokenLabel(props: IAccountTokenLabelProps) {
  return (
    <ClipboardLabel
      value={props.accountToken}
      displayValue={formatAccountToken(props.accountToken)}
      className={props.className}
    />
  );
}
