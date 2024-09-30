export function formatAccountNumber(accountNumber: string) {
  const parts =
    accountNumber.replace(/\s+| /g, '').substring(0, 16).match(new RegExp('.{1,4}', 'g')) || [];
  return parts.join(' ');
}
