import { Text } from '../../../../../lib/components';
import { useSelector } from '../../../../../redux/store';
import AccountNumberLabel from '../../../../AccountNumberLabel';

export function AccountNumberRow() {
  const accountNumber = useSelector((state) => state.account.accountNumber);
  return (
    <Text variant="bodySmallSemibold" as={AccountNumberLabel} accountNumber={accountNumber || ''} />
  );
}
