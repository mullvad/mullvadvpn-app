import { useStatusText } from './hooks';

export function StatusText() {
  const status = useStatusText();
  return <>{status}</>;
}
